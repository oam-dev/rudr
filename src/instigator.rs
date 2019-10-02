use failure::Error;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::{api::Object, api::PatchParams, api::RawApi, api::Void, client::APIClient};
use log::{debug, error, info};
use serde_json::json;
use std::collections::BTreeMap;

use crate::{
    lifecycle::Phase,
    schematic::{
        component::Component,
        component_instance::KubeComponentInstance,
        configuration::ApplicationConfiguration,
        configuration::ComponentConfiguration,
        parameter::{resolve_parameters, resolve_values, ParameterValue},
        traits::{self, Autoscaler, Empty, HydraTrait, Ingress, ManualScaler, TraitBinding},
        HydraStatus, Status,
    },
    workload_type::{
        self, CoreWorkloadType, ReplicatedService, ReplicatedTask, ReplicatedWorker,
        SingletonService, SingletonTask, SingletonWorker, WorkloadMetadata, HYDRA_API_VERSION,
    },
};

pub const CONFIG_GROUP: &str = "core.hydra.io";
pub const CONFIG_VERSION: &str = "v1alpha1";

pub const CONFIG_CRD: &str = "applicationconfigurations";
pub const COMPONENT_CRD: &str = "componentschematics";
pub const TRAIT_CRD: &str = "traits";
pub const SCOPE_CRD: &str = "scopes";
pub const COMPONENT_RECORD_ANNOTATION: &str = "component_record_annotation";

/// Type alias for the results that all instantiation operations return
pub type InstigatorResult = Result<(), Error>;
type OpResource = Object<ApplicationConfiguration, Status>;
type ParamMap = BTreeMap<String, serde_json::Value>;

/// This error is returned when a component cannot be found.
#[derive(Fail, Debug)]
#[fail(display = "Component {} not found", name)]
pub struct ComponentNotFoundError {
    name: String,
}

/// An Instigator takes an inbound object and manages the reconcilliation with the desired objects.
///
/// Any given Component may, underneath the hood, be composed of multiple Kubernetes objects.
/// For example, a ReplicableService will create (at least) a Deployment and a Service
/// (and probably a Secret or ConfigMap as well as some RBACs). The individual pieces are
/// managed by their respective WorkloadType. The Instigator's job is to read a component,
/// and then delegate to the correct WorkloadType.
///
/// Traits and Scopes are application configuration. As such, it is not the responsibility of
/// the WorkloadType to manage those. Thus, after delegating work to the WorkloadType, the
/// Instigator will examine the Traits and Scopes requirements, and delegate those
/// processes to the appropriate Scope or TraitImpl.
///
/// (Terminological note: Hydra Traits are distinct from Rust traits. TraitImpl is the
/// Rust trait that represents a Hydra Trait)
///
/// Instigators know how to deal with the following operations:
/// - Add
/// - Modify
/// - Delete
#[derive(Clone)]
pub struct Instigator {
    client: APIClient,
    //cache: Reflector<Component, Status>,
    namespace: String,
}

/// Alias for a Kubernetes wrapper on a component.
type KubeComponent = Object<Component, Void>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ComponentRecord {
    pub config: ComponentConfiguration,
    pub version: String,
}
pub type RecordAnnotation = BTreeMap<String, ComponentRecord>;

impl Instigator {
    /// Create a new instigator
    ///
    /// An instigator uses the reflector as a cache of Components, and will use the API client
    /// for creating and managing the component implementation.
    pub fn new(client: kube::client::APIClient, namespace: String) -> Self {
        Instigator { client, namespace }
    }

    /// The workhorse for Instigator.
    /// This will execute only Add, Modify, and Delete phases.
    fn exec(&self, event: OpResource, mut phase: Phase) -> InstigatorResult {
        // TODO:
        // - Resolve scope bindings
        let name = event.metadata.name.clone();
        let owner_ref = config_owner_reference(name.clone(), event.metadata.uid.clone())?;

        let record_ann = event.metadata.annotations.get(COMPONENT_RECORD_ANNOTATION);
        let mut last_components = get_record_annotation(record_ann)?;
        let mut new_components: BTreeMap<String, ComponentRecord> = BTreeMap::new();
        let mut component_updated = false;
        for component in event.clone().spec.components.unwrap_or_else(|| vec![]) {
            let record = last_components
                .get_mut(component.instance_name.as_str())
                .cloned();
            let comp_def: KubeComponent = get_component_def(
                self.namespace.clone(),
                component.name.clone(),
                self.client.clone(),
            )?;
            //check last components in every component loop
            let new_record = &ComponentRecord {
                version: comp_def.clone().metadata.resourceVersion.unwrap(),
                config: component.clone(),
            };
            //remove the component in last and add to new, when we finish the component loop mentioned in event,
            //we'll delete the left in last_components.
            last_components.remove(component.instance_name.as_str());
            new_components.insert(component.instance_name.clone(), new_record.to_owned());
            if !check_diff(record.clone(), new_record) {
                continue;
            }
            // record exists means component exists so event is just modify
            // while record is none means component don't exist so event is Add
            if record.is_some() && phase == Phase::Add {
                phase = Phase::Modify
            } else if record.is_none() && phase == Phase::Modify {
                phase = Phase::Add
            }

            component_updated = true;
            // Resolve parameters
            let parent = get_values(event.spec.parameter_values.clone());
            let child = get_values(component.parameter_values.clone());
            let merged_vals = resolve_values(child, parent.clone())?;
            let params = resolve_parameters(comp_def.spec.parameters.clone(), merged_vals)?;

            let inst_name = component.instance_name.clone();
            let new_owner_ref = match phase {
                Phase::Add => Some(self.create_component_instance(
                    component.name.clone(),
                    inst_name.clone(),
                    owner_ref.clone(),
                )?),
                Phase::Modify => {
                    let ownref = self.component_instance_owner_reference(
                        component.name.clone(),
                        inst_name.clone(),
                    );
                    if ownref.is_err() {
                        let e = ownref.unwrap_err().to_string();
                        if !e.contains("NotFound") {
                            // Wrap the error to make it clear where we failed
                            // During deletion, this might indicate that something else
                            // remove the component instance.
                            return Err(format_err!(
                                "{:?} on {}: {}",
                                phase.clone(),
                                inst_name.clone(),
                                e
                            ));
                        }
                        Some(self.create_component_instance(
                            component.name.clone(),
                            inst_name.clone(),
                            owner_ref.clone(),
                        )?)
                    } else {
                        Some(ownref.unwrap())
                    }
                }
                _ => None,
            };

            // Instantiate components
            let workload = self.load_workload_type(
                name.clone(),
                inst_name.clone(),
                &comp_def,
                &params,
                new_owner_ref.clone(),
            )?;
            // Load all of the traits related to this component.
            let mut trait_manager = TraitManager {
                config_name: name.clone(),
                instance_name: inst_name.clone(),
                component: component.clone(),
                parent_params: parent.clone(),
                owner_ref: new_owner_ref.clone(),
                workload_type: comp_def.spec.workload_type.clone(),
                traits: vec![], // Always starts empty.
            };
            trait_manager.load_traits()?;

            match phase {
                Phase::Add => {
                    info!("Adding component {}", component.name.clone());
                    trait_manager.exec(
                        self.namespace.as_str(),
                        self.client.clone(),
                        Phase::PreAdd,
                    )?;
                    workload.add()?;
                    trait_manager.exec(self.namespace.as_str(), self.client.clone(), Phase::Add)?;
                }
                Phase::Modify => {
                    info!("Modifying component {}", component.name.clone());
                    trait_manager.exec(
                        self.namespace.as_str(),
                        self.client.clone(),
                        Phase::PreModify,
                    )?;
                    workload.modify()?;
                    trait_manager.exec(
                        self.namespace.as_str(),
                        self.client.clone(),
                        Phase::Modify,
                    )?;
                }
                Phase::Delete => {
                    info!("Deleting component {}", component.name.clone());
                    trait_manager.exec(
                        self.namespace.as_str(),
                        self.client.clone(),
                        Phase::PreDelete,
                    )?;
                    // we leave owner reference to do delete work, so we don't need to invoke delete function here.
                }
                _ => {
                    return Err(format_err!(
                        "Illegal phase: only Add, Modify, and Delete are supported here"
                    ))
                }
            }
        }

        // delete the component left
        for component_record in last_components.values() {
            let component = component_record.config.clone();
            component_updated = true;
            //FIXME: if component is not found, what can we do?
            let comp_def: KubeComponent = get_component_def(
                self.namespace.clone(),
                component.name.clone(),
                self.client.clone(),
            )?;
            // Resolve parameters
            let parent = get_values(event.spec.parameter_values.clone());

            let inst_name = component.instance_name.clone();
            // Load all of the traits related to this component.
            let mut trait_manager = TraitManager {
                config_name: name.clone(),
                instance_name: inst_name.clone(),
                component: component.clone(),
                parent_params: parent.clone(),
                owner_ref: None,
                workload_type: comp_def.spec.workload_type.clone(),
                traits: vec![], // Always starts empty.
            };
            trait_manager.load_traits()?;

            info!("Deleting component {}", component.name.clone());
            //The reason for this is that we do not require that traits be deployed only in-cluster.
            //For example, a trait could create an object storage bucket or work with an external API service.
            //So we want to give them a chance to react to a deletion event.
            trait_manager.exec(
                self.namespace.as_str(),
                self.client.clone(),
                Phase::PreDelete,
            )?;
            //delete component instance and let owner_reference to delete real resource
            self.delete_component_instance(component.name.clone(), inst_name.clone())?;
        }
        // if no component was updated or this is an delete phase, just return without status change.
        if !component_updated || phase == Phase::Delete {
            return Ok(());
        }

        let new_record = serde_json::to_string(&new_components)?;
        let mut annotation = event.metadata.annotations.clone();
        annotation.insert(COMPONENT_RECORD_ANNOTATION.to_string(), new_record);
        let status = HydraStatus::new(Some(phase.to_string()));
        let mut new_event = event.clone();
        new_event.status = Some(Some(status));
        new_event.metadata.annotations = annotation;
        debug!("spec: {:?}, status: {:?}", new_event.spec, new_event.status);
        let config_resource = RawApi::customResource(CONFIG_CRD)
            .version(CONFIG_VERSION)
            .group(CONFIG_GROUP)
            .within(&self.namespace);

        let patch_params = PatchParams::default();
        let req = config_resource.patch(
            &event.metadata.name,
            &patch_params,
            serde_json::to_vec(&new_event)?,
        )?;
        let o = self
            .client
            .request::<Object<ApplicationConfiguration, Status>>(req)?;
        info!("Patched status {:?} for {}", o.status, o.metadata.name);
        Ok(())
    }

    /// Create new Kubernetes objects based on this config.
    pub fn add(&self, event: OpResource) -> InstigatorResult {
        self.exec(event, Phase::Add)
    }
    /// Modify existing Kubernetes objects based on config and workload type.
    pub fn modify(&self, event: OpResource) -> InstigatorResult {
        self.exec(event, Phase::Modify)
    }
    /// Delete the Kubernetes objects associated with this config.
    pub fn delete(&self, event: OpResource) -> InstigatorResult {
        self.exec(event, Phase::Delete)
    }

    fn load_workload_type(
        &self,
        config_name: String,
        instance_name: String,
        comp: &KubeComponent,
        params: &ParamMap,
        owner_ref: Option<Vec<meta::OwnerReference>>,
    ) -> Result<CoreWorkloadType, Error> {
        info!("Looking up {}", config_name);
        let meta = WorkloadMetadata {
            name: config_name,
            instance_name,
            component_name: comp.metadata.name.clone(),
            annotations: Some(comp.metadata.annotations.clone()),
            namespace: self.namespace.clone(),
            definition: comp.spec.clone(),
            client: self.client.clone(),
            params: params.clone(),
            owner_ref,
        };
        match comp.spec.workload_type.as_str() {
            workload_type::SERVER_NAME => {
                let rs = ReplicatedService { meta };
                Ok(CoreWorkloadType::ReplicatedServiceType(rs))
            }
            workload_type::SINGLETON_SERVER_NAME => {
                let sing = SingletonService { meta };
                Ok(CoreWorkloadType::SingletonServiceType(sing))
            }
            workload_type::SINGLETON_TASK_NAME => {
                let task = SingletonTask { meta };
                Ok(CoreWorkloadType::SingletonTaskType(task))
            }
            workload_type::TASK_NAME => {
                let task = ReplicatedTask {
                    meta,
                    replica_count: Some(1), // Every(1) needs Some(1) to love.
                };
                Ok(CoreWorkloadType::ReplicatedTaskType(task))
            }
            workload_type::SINGLETON_WORKER => {
                let wrkr = SingletonWorker { meta };
                Ok(CoreWorkloadType::SingletonWorkerType(wrkr))
            }
            workload_type::WORKER_NAME => {
                let worker = ReplicatedWorker {
                    meta,
                    replica_count: Some(1), // Every(1) needs Some(1) to love.
                };
                Ok(CoreWorkloadType::ReplicatedWorkerType(worker))
            }
            _ => Err(format_err!(
                "workloadType {} is unknown",
                comp.spec.workload_type
            )),
        }
    }

    fn delete_component_instance(
        &self,
        component_name: String,
        instance_name: String,
    ) -> InstigatorResult {
        let name = combine_name(component_name, instance_name);
        let pp = kube::api::DeleteParams::default();
        let crd_req = RawApi::customResource("componentinstances")
            .group(CONFIG_GROUP)
            .version(CONFIG_VERSION)
            .within(self.namespace.as_str());
        let req = crd_req.delete(name.as_str(), &pp)?;
        if let Err(e) = self.client.request_status::<KubeComponentInstance>(req) {
            if e.to_string().contains("NotFound") {
                return Ok(());
            }
            return Err(e.into());
        }
        Ok(())
    }

    fn create_component_instance(
        &self,
        component_name: String,
        instance_name: String,
        owner: meta::OwnerReference,
    ) -> Result<Vec<meta::OwnerReference>, Error> {
        let name = combine_name(component_name, instance_name);
        let pp = kube::api::PostParams::default();
        let crd_req = RawApi::customResource("componentinstances")
            .group(CONFIG_GROUP)
            .version(CONFIG_VERSION)
            .within(self.namespace.as_str());
        let comp_inst = json!({
            "apiVersion": HYDRA_API_VERSION,
            "kind": "ComponentInstance",
            "metadata": {
                "name": name.clone(),
                "ownerReferences": [{
                    "apiVersion": HYDRA_API_VERSION,
                    "kind": "ApplicationConfiguration",
                    "controller": true,
                    "blockOwnerDeletion": true,
                    "name": owner.name.clone(),
                    "uid": owner.uid.clone(),
                }]
            },
            "spec": {
                "traits": []
            }
        });

        let req = crd_req.create(&pp, serde_json::to_vec(&comp_inst)?)?;
        let res: KubeComponentInstance = match self.client.request(req) {
            Ok(res) => res,
            Err(e) => {
                if let Some(api_err) = e.api_error() {
                    if api_err.reason == "AlreadyExists" {
                        let req = crd_req.get(name.as_str())?;
                        self.client.request(req)?
                    } else {
                        return Err(e.into());
                    }
                } else {
                    return Err(e.into());
                }
            }
        };

        if res.metadata.uid.is_none() {
            return Err(format_err!("UID was not set on component instance"));
        }
        info!("UID: {}", res.metadata.uid.clone().unwrap());

        let new_owner = meta::OwnerReference {
            api_version: HYDRA_API_VERSION.into(),
            kind: "ComponentInstance".into(),
            uid: res.metadata.uid.unwrap(),
            controller: Some(true),
            block_owner_deletion: Some(true),
            name,
        };
        Ok(vec![new_owner])
    }

    fn component_instance_owner_reference(
        &self,
        component_name: String,
        instance_name: String,
    ) -> Result<Vec<meta::OwnerReference>, Error> {
        let name = combine_name(component_name, instance_name);
        let crd_req = RawApi::customResource("componentinstances")
            .group(CONFIG_GROUP)
            .version(CONFIG_VERSION)
            .within(self.namespace.as_str());
        let req = crd_req.get(name.as_str())?;
        let res: KubeComponentInstance = self.client.request(req)?;

        let owner = meta::OwnerReference {
            api_version: HYDRA_API_VERSION.into(),
            kind: "ComponentInstance".into(),
            uid: res.metadata.uid.unwrap(),
            controller: Some(true),
            block_owner_deletion: Some(true),
            name: res.metadata.name,
        };
        Ok(vec![owner])
    }
}

/// combine_name combine component name with instance_name,
/// so we won't afraid different components using same instance_name   
pub fn combine_name(component_name: String, instance_name: String) -> String {
    component_name + "-" + instance_name.as_str()
}

/// Build an owner reference for the given parent UID of kind Configuration.
pub fn config_owner_reference(
    parent_name: String,
    parent_uid: Option<String>,
) -> Result<meta::OwnerReference, Error> {
    match parent_uid {
        Some(uid) => {
            let owner_ref = meta::OwnerReference {
                api_version: HYDRA_API_VERSION.into(),
                kind: "ApplicationConfiguration".into(),
                uid,
                controller: Some(true),
                block_owner_deletion: Some(true),
                name: parent_name.clone(),
            };
            Ok(owner_ref)
        }
        None => Err(format_err!(
            "Mysteriously, no UID was created. Ancient version of Kubernetes?"
        )),
    }
}

/// get_record_annotation json unmarshal component record from annotation
pub fn get_record_annotation(
    records_ann: Option<&String>,
) -> Result<RecordAnnotation, serde_json::error::Error> {
    match records_ann {
        Some(str) => serde_json::from_str(str.as_str()),
        None => Ok(BTreeMap::new()),
    }
}

pub fn check_diff(old: Option<ComponentRecord>, new: &ComponentRecord) -> bool {
    match old {
        None => true,
        Some(oldr) => oldr != new.clone(),
    }
}

pub fn get_component_def(
    namespace: String,
    comp_name: String,
    client: APIClient,
) -> Result<KubeComponent, Error> {
    let component_resource = RawApi::customResource(COMPONENT_CRD)
        .version("v1alpha1")
        .group("core.hydra.io")
        .within(&namespace);
    let comp_def_req = component_resource.get(comp_name.as_str())?;
    let comp_def: KubeComponent = client.request::<KubeComponent>(comp_def_req)?;
    Ok(comp_def)
}

pub fn get_values(values: Option<Vec<ParameterValue>>) -> Vec<ParameterValue> {
    values.or_else(|| Some(vec![])).unwrap()
}

// TraitManager maps a component to its traits, and handles trait lifecycle.
//
// Each component configuration is assigned a trait manager. That trait manager
// can load all of the associated traits, and then executed phases for each of
// the traits.
struct TraitManager {
    config_name: String,
    instance_name: String,
    component: ComponentConfiguration,
    parent_params: Vec<ParameterValue>,
    owner_ref: Option<Vec<meta::OwnerReference>>,
    workload_type: String,

    traits: Vec<HydraTrait>,
}

impl TraitManager {
    fn load_traits(&mut self) -> Result<(), Error> {
        let mut traits: Vec<HydraTrait> = vec![];
        for t in self.component.traits.as_ref().unwrap_or(&vec![]).iter() {
            // Load all of the traits into the manager.
            let imp = self.load_trait(&t)?;
            traits.push(imp);
        }
        self.traits = traits;
        Ok(())
    }
    fn load_trait(&self, binding: &TraitBinding) -> Result<HydraTrait, Error> {
        let trait_values = resolve_values(
            binding.parameter_values.clone().unwrap_or_else(|| vec![]),
            self.parent_params.clone(),
        )?;
        debug!("Trait binding params: {:?}", &binding.parameter_values);
        match binding.name.as_str() {
            traits::INGRESS => {
                let ing = Ingress::from_params(
                    self.config_name.clone(),
                    self.instance_name.clone(),
                    self.component.name.clone(),
                    trait_values,
                    self.owner_ref.clone(),
                );
                Ok(HydraTrait::Ingress(ing))
            }
            traits::AUTOSCALER => {
                let auto = Autoscaler::from_params(
                    self.config_name.clone(),
                    self.instance_name.clone(),
                    self.component.name.clone(),
                    trait_values,
                    self.owner_ref.clone(),
                );
                Ok(HydraTrait::Autoscaler(auto))
            }
            traits::MANUAL_SCALER => {
                let scaler = ManualScaler::from_params(
                    self.config_name.clone(),
                    self.instance_name.clone(),
                    self.component.name.clone(),
                    trait_values,
                    self.owner_ref.clone(),
                    self.workload_type.clone(),
                );
                Ok(HydraTrait::ManualScaler(scaler))
            }
            // Empty is a debugging tool for checking whether the traits system is functioning independently of
            // its environment.
            traits::EMPTY => {
                let empty = Empty {};
                Ok(HydraTrait::Empty(empty))
            }
            _ => Err(format_err!("unknown trait {}", binding.name)),
        }
    }
    fn exec(&self, ns: &str, client: APIClient, phase: Phase) -> Result<(), Error> {
        for imp in &self.traits {
            // At the moment, we don't return an error if a trait fails.
            let res = imp.exec(ns, client.clone(), phase.clone());
            if res.is_err() {
                error!(
                    "Trait phase {:?} failed for {}: {}",
                    phase,
                    self.config_name.as_str(),
                    res.unwrap_err()
                );
            }
        }
        Ok(())
    }
}
