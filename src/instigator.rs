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
        configuration::ComponentConfiguration,
        configuration::OperationalConfiguration,
        configuration::ScopeBinding,
        parameter::{resolve_parameters, resolve_values, ParameterValue},
        scopes::{Health, HydraScope},
        traits::{Autoscaler, Empty, HydraTrait, Ingress, ManualScaler, TraitBinding},
        HydraStatus, Status,
    },
    workload_type::{
        CoreWorkloadType, ReplicatedService, ReplicatedTask, ReplicatedWorker, SingletonService,
        SingletonTask, SingletonWorker, WorkloadMetadata, HYDRA_API_VERSION,
    },
};

/// Type alias for the results that all instantiation operations return
pub type InstigatorResult = Result<(), Error>;
type OpResource = Object<OperationalConfiguration, Status>;
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
/// Traits and Scopes are operational configuration. As such, it is not the responsibility of
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

impl Instigator {
    /// Create a new instigator
    ///
    /// An instigator uses the reflector as a cache of Components, and will use the API client
    /// for creating and managing the component implementation.
    pub fn new(client: kube::client::APIClient, ns: String) -> Self {
        Instigator {
            client: client,
            namespace: ns,
        }
    }

    /// The workhorse for Instigator.
    /// This will execute only Add, Modify, and Delete phases.
    fn exec(&self, event: OpResource, phase: Phase) -> InstigatorResult {
        // TODO:
        // - Resolve scope bindings
        // First check if there is not allowed overlap
        let scopes = load_scopes(event.spec.clone().scopes)?;
        let name = event.metadata.name.clone();
        let owner_ref = config_owner_reference(name.clone(), event.metadata.uid.clone())?;

        for component in event.spec.clone().components.unwrap_or(vec![]) {
            let component_resource = RawApi::customResource("components")
                .version("v1alpha1")
                .group("core.hydra.io")
                .within(&self.namespace);
            let comp_def_req = component_resource.get(component.name.as_str())?;
            let comp_def: KubeComponent = self.client.request::<KubeComponent>(comp_def_req)?;

            // Resolve parameters
            let parent = event
                .spec
                .parameter_values
                .clone()
                .or(Some(vec![]))
                .unwrap();
            let child = component.parameter_values.clone().or(Some(vec![])).unwrap();
            let merged_vals = resolve_values(child, parent.clone())?;
            let params = resolve_parameters(comp_def.spec.parameters.clone(), merged_vals)?;

            let inst_name = component.instance_name.clone();
            let new_owner_ref = match phase {
                Phase::Add => {
                    self.create_component_instance(inst_name.clone(), owner_ref.clone())?
                }
                _ => {
                    let ownref = self.component_instance_owner_reference(inst_name.as_str());
                    if ownref.is_err() {
                        // Wrap the error to make it clear where we failed
                        // During deletion, this might indicate that something else
                        // remove the component instance.
                        return Err(format_err!(
                            "{:?} on {}: {}",
                            phase.clone(),
                            inst_name.clone(),
                            ownref.unwrap_err()
                        ));
                    }
                    ownref.unwrap()
                }
            };

            //TODO: Check if network scopes need to apply on workload
            // TODO: add component to watch
            for sc in &scopes {
                let type_name = sc.scope_type();
                if type_name == "core.hydra.io/v1alpha1.Health" {
                    println!("add this to watch loop");
                }
            }

            // Instantiate components
            let workload = self.load_workload_type(
                name.clone(),
                inst_name.clone(),
                &comp_def,
                &params,
                Some(new_owner_ref.clone()),
            )?;
            // Load all of the traits related to this component.
            let mut trait_manager = TraitManager {
                config_name: name.clone(),
                instance_name: inst_name.clone(),
                component: component.clone(),
                parent_params: parent.clone(),
                owner_ref: Some(new_owner_ref.clone()),
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
                    workload.delete()?;
                    trait_manager.exec(
                        self.namespace.as_str(),
                        self.client.clone(),
                        Phase::Delete,
                    )?;
                }
                _ => {
                    return Err(format_err!(
                        "Illegal phase: only Add, Modify, and Delete are supported here"
                    ))
                }
            }
        }
        match event.status.clone() {
            Some(s) => match s.clone() {
                Some(hs) => {
                    if hs.phase.is_some() && hs.phase.unwrap() == phase.to_string() {
                        return Ok(());
                    }
                }
                None => {}
            },
            None => {}
        }

        let status = HydraStatus::new(Some(phase.to_string()));
        let mut new_event = event.clone();
        new_event.status = Some(Some(status));
        debug!("{:?},{:?}", new_event.spec, new_event.status);

        let config_resource = RawApi::customResource("configurations")
            .version("v1alpha1")
            .group("core.hydra.io")
            .within(&self.namespace);

        let patch_params = PatchParams::default();
        let req = config_resource.patch(
            &event.metadata.name,
            &patch_params,
            serde_json::to_vec(&new_event)?,
        )?;
        let o = self
            .client
            .request::<Object<OperationalConfiguration, Status>>(req)?;
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
        owner: Option<Vec<meta::OwnerReference>>,
    ) -> Result<CoreWorkloadType, Error> {
        info!("Looking up {}", config_name);
        let meta = WorkloadMetadata {
            name: config_name,
            instance_name: instance_name,
            component_name: comp.metadata.name.clone(),
            namespace: self.namespace.clone(),
            definition: comp.spec.clone(),
            client: self.client.clone(),
            params: params.clone(),
            owner_ref: owner,
        };
        match comp.spec.workload_type.as_str() {
            // This one is DEPRECATED
            "core.hydra.io/v1alpha1.ReplicatedService" => {
                let rs = ReplicatedService { meta: meta };
                Ok(CoreWorkloadType::ReplicatedServiceType(rs))
            }
            // DEPRECATED
            "core.hydra.io/v1alpha1.Singleton" => {
                let sing = SingletonService { meta: meta };
                Ok(CoreWorkloadType::SingletonType(sing))
            }
            "core.hydra.io/v1alpha1.Service" => {
                let rs = ReplicatedService { meta: meta };
                Ok(CoreWorkloadType::ReplicatedServiceType(rs))
            }
            "core.hydra.io/v1alpha1.SingletonService" => {
                let sing = SingletonService { meta: meta };
                Ok(CoreWorkloadType::SingletonType(sing))
            }
            "core.hydra.io/v1alpha1.SingletonTask" => {
                let task = SingletonTask { meta: meta };
                Ok(CoreWorkloadType::SingletonTaskType(task))
            }
            "core.hydra.io/v1alpha1.Task" => {
                let task = ReplicatedTask {
                    meta: meta,
                    replica_count: Some(1), // Every(1) needs Some(1) to love.
                };
                Ok(CoreWorkloadType::ReplicatedTaskType(task))
            }
            "core.hydra.io/v1alpha1.SingletonWorker" => {
                let wrkr = SingletonWorker { meta: meta };
                Ok(CoreWorkloadType::SingletonWorkerType(wrkr))
            }
            "core.hydra.io/v1alpha1.Worker" => {
                let worker = ReplicatedWorker {
                    meta: meta,
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

    fn create_component_instance(
        &self,
        name: String,
        owner: meta::OwnerReference,
    ) -> Result<Vec<meta::OwnerReference>, Error> {
        let pp = kube::api::PostParams::default();
        let crd_req = RawApi::customResource("componentinstances")
            .group("core.hydra.io")
            .version("v1alpha1")
            .within(self.namespace.as_str());
        let comp_inst = json!({
            "apiVersion": "core.hydra.io/v1alpha1",
            "kind": "ComponentInstance",
            "metadata": {
                "name": name.clone(),
                "ownerReferences": [{
                    "apiVersion": HYDRA_API_VERSION,
                    "kind": "Configuration",
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
        let res: KubeComponentInstance = self.client.request(req)?;

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
            name: name,
        };
        Ok(vec![new_owner])
    }

    fn component_instance_owner_reference(
        &self,
        name: &str,
    ) -> Result<Vec<meta::OwnerReference>, Error> {
        let crd_req = RawApi::customResource("componentinstances")
            .group("core.hydra.io")
            .version("v1alpha1")
            .within(self.namespace.as_str());
        let req = crd_req.get(name)?;
        let res: KubeComponentInstance = self.client.request(req)?;
        debug!("get instance {:?}", res.spec);

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

/// Build an owner reference for the given parent UID of kind Configuration.
pub fn config_owner_reference(
    parent_name: String,
    parent_uid: Option<String>,
) -> Result<meta::OwnerReference, Error> {
    match parent_uid {
        Some(uid) => {
            let owner_ref = meta::OwnerReference {
                api_version: HYDRA_API_VERSION.into(),
                kind: "Configuration".into(),
                uid: uid,
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

/// Load application scopes from scope_bindings
pub fn load_scopes(
    scope_bindings: Option<Vec<ScopeBinding>>,
) -> Result<Vec<HydraScope>, failure::Error> {
    let mut scopes: Vec<HydraScope> = vec![];
    let mut scope_overlap = BTreeMap::new();
    if scope_bindings.is_none() {
        return Ok(scopes);
    }
    for sc in scope_bindings.unwrap() {
        let scope = load_scope(&sc)?;
        if !scope.allow_overlap() {
            if scope_overlap.get(&sc.scope_type).is_some() {
                return Err(format_err!(
                    "scope {} {} do not allow overlap",
                    sc.name,
                    sc.scope_type
                ));
            }
        }
        scopes.insert(scopes.len(), scope);
        scope_overlap.insert(sc.scope_type, true);
    }
    Ok(scopes)
}

// load_scope load scope from k8s crd
// NOTE: this is a temporary solution just return core scope here
fn load_scope(binding: &ScopeBinding) -> Result<HydraScope, failure::Error> {
    debug!("Scope binding params: {:?}", &binding.parameter_values);
    match binding.name.as_str() {
        /*
        "core.hydra.io/v1alpha1.Network" => Ok(HydraScope::Network(Network::from_params(
            binding.name.clone(),
            binding.parameter_values,
        ))),
        */
        "core.hydra.io/v1alpha1.Health" => Ok(HydraScope::Health(Health::from_params(
            binding.name.clone(),
            binding.parameter_values.clone(),
        ))),
        _ => Err(format_err!(
            "unknown scope {} type {}",
            binding.name,
            binding.scope_type
        )),
    }
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
            binding.parameter_values.clone().unwrap_or(vec![]),
            self.parent_params.clone(),
        )?;
        debug!("Trait binding params: {:?}", &binding.parameter_values);
        match binding.name.as_str() {
            "ingress" => {
                let ing = Ingress::from_params(
                    self.config_name.clone(),
                    self.instance_name.clone(),
                    self.component.name.clone(),
                    trait_values,
                    self.owner_ref.clone(),
                );
                Ok(HydraTrait::Ingress(ing))
            }
            "autoscaler" => {
                let auto = Autoscaler::from_params(
                    self.config_name.clone(),
                    self.instance_name.clone(),
                    self.component.name.clone(),
                    trait_values,
                    self.owner_ref.clone(),
                );
                Ok(HydraTrait::Autoscaler(auto))
            }
            "manual-scaler" => {
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
            "empty" => {
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
