use failure::Error;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::{api::Api, api::Object, api::PatchParams, api::RawApi, api::Void, client::APIClient};
use log::{debug, error, info, warn};
use serde_json::json;
use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::ObjectReference;

use crate::schematic::variable::Variable;
use crate::{
    kube_event,
    lifecycle::Phase,
    schematic::{
        component::Component,
        component_instance::KubeComponentInstance,
        configuration::{ApplicationConfiguration, ComponentConfiguration, ScopeBinding},
        parameter::{resolve_parameters, resolve_values, ParameterValue},
        scopes::{self, Health, Network, OAMScope},
        variable::{get_variable_values, resolve_variables},
        OAMStatus,
    },
    trait_manager::TraitManager,
    workload_type::{
        self, CoreWorkloadType, ExtendedWorkloadType, ReplicatedServer, ReplicatedTask,
        ReplicatedWorker, SingletonServer, SingletonTask, SingletonWorker, WorkloadMetadata,
        WorkloadType, OAM_API_VERSION,
    },
};

pub const CONFIG_GROUP: &str = "core.oam.dev";
pub const CONFIG_VERSION: &str = "v1alpha1";

pub const CONFIG_CRD: &str = "applicationconfigurations";
pub const COMPONENT_CRD: &str = "componentschematics";
pub const TRAIT_CRD: &str = "traits";
pub const SCOPE_CRD: &str = "scopes";
pub const COMPONENT_RECORD_ANNOTATION: &str = "component_record_annotation";

/// Type alias for the results that all instantiation operations return
pub type InstigatorResult = Result<(), Error>;
pub type OpResource = Object<ApplicationConfiguration, OAMStatus>;
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
/// For example, a Server will create (at least) a Deployment and a Service
/// (and probably a Secret or ConfigMap as well as some RBACs). The individual pieces are
/// managed by their respective WorkloadType. The Instigator's job is to read a component,
/// and then delegate to the correct WorkloadType.
///
/// Traits and Scopes are application configuration. As such, it is not the responsibility of
/// the WorkloadType to manage those. Thus, after delegating work to the WorkloadType, the
/// Instigator will examine the Traits and Scopes requirements, and delegate those
/// processes to the appropriate Scope or TraitImpl.
///
/// (Terminological note: Open Application Model Traits are distinct from Rust traits.
/// TraitImpl is the Rust trait that represents an OAM Trait)
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
    pub event_handler: kube_event::Event,
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
        Instigator {
            client: client.clone(),
            namespace: namespace.clone(),
            event_handler: kube_event::Event::new(client, namespace),
        }
    }

    pub fn sync_status(&self, event: OpResource) -> InstigatorResult {
        let mut component_status = BTreeMap::new();
        let name = event.metadata.name.clone();
        let record_ann = event.metadata.annotations.get(COMPONENT_RECORD_ANNOTATION);
        let mut last_components = get_record_annotation(record_ann)?;
        let mut has_diff = false;
        for component in event.clone().spec.components.unwrap_or_else(|| vec![]) {
            let comp_def: KubeComponent = get_component_def(
                self.namespace.clone(),
                component.component_name.clone(),
                self.client.clone(),
            )?;

            let new_record = &ComponentRecord {
                version: comp_def.clone().metadata.resourceVersion.unwrap(),
                config: component.clone(),
            };
            let record = last_components
                .get_mut(component.instance_name.as_str())
                .cloned();
            last_components.remove(component.instance_name.as_str());
            // we can't update status when there is update not finished yet.
            if check_diff(record.clone(), new_record) {
                has_diff = true;
                continue;
            }
            // Resolve variables/parameters
            let variables = event.spec.variables.clone().unwrap_or_else(|| vec![]);
            let parent = get_variable_values(Some(variables.clone()));

            let child = component
                .parameter_values
                .clone()
                .map(|values| resolve_variables(values, variables))
                .unwrap_or_else(|| Ok(vec![]))?;

            let params = resolve_parameters(
                comp_def.spec.parameters.clone(),
                resolve_values(child, vec![])?,
            )?;

            let inst_name = component.instance_name.clone();

            let workload_meta = self.get_workload_meta(
                name.clone(),
                inst_name.clone(),
                &comp_def,
                &params,
                None,
                "StatusCheckLoop".to_string(),
            );
            // Instantiate components
            let workload = self.load_workload_type(&comp_def, workload_meta)?;
            let mut status = workload.status()?;
            debug!(
                "StatusCheckLoop: Sync component {}, got status {:?}",
                component.component_name.clone(),
                status.clone()
            );
            let mut health_state = "healthy".to_string();
            for (_, v) in status.clone() {
                if v != "running" && v != "created" && v != "succeeded" {
                    health_state = "unhealthy".to_string();
                    break;
                }
            }
            self.component_instance_set_status(
                component.component_name.clone(),
                inst_name.clone(),
                health_state,
            )?;
            // Load all of the traits related to this component.
            let mut trait_manager = TraitManager {
                config_name: name.clone(),
                instance_name: inst_name.clone(),
                component: component.clone(),
                parent_params: parent.clone(),
                owner_ref: None,
                workload_type: comp_def.spec.workload_type.clone(),
                traits: vec![], // Always starts empty.
                component_schematic: comp_def.spec.clone(),
            };
            trait_manager.load_traits()?;
            if let Some(trait_status) =
                trait_manager.status(self.namespace.as_str(), self.client.clone())
            {
                for (key, state) in trait_status {
                    status.insert(key, state);
                }
            };
            component_status.insert(component.component_name.clone(), status.clone());
        }
        //we won't update status if there's any update
        if has_diff || !last_components.is_empty() {
            info!(
                "StatusCheckLoop: skip update status for {}, wait for the mainControlLoop to execute",
                event.metadata.name.clone()
            );
            return Ok(());
        }
        self.retry_patch_status(
            event.clone(),
            Some(OAMStatus::new(
                Some("synced".to_string()),
                Some(component_status),
            )),
            None,
            "StatusCheckLoop".to_string(),
        )
    }

    pub fn retry_patch_status(
        &self,
        event: OpResource,
        status: Option<OAMStatus>,
        annotation: Option<BTreeMap<String, String>>,
        controlled_by: String,
    ) -> InstigatorResult {
        let config_resource: Api<Object<ApplicationConfiguration, OAMStatus>> =
            Api::customResource(self.client.clone(), CONFIG_CRD)
                .version(CONFIG_VERSION)
                .group(CONFIG_GROUP)
                .within(&self.namespace);
        let mut new_event = event.clone();
        loop {
            new_event.status = status.clone();
            if let Some(newann) = annotation.clone() {
                new_event.metadata.annotations = newann;
            }
            let patch_params = PatchParams::default();
            match config_resource.patch(
                &event.metadata.name,
                &patch_params,
                serde_json::to_vec(&new_event)?,
            ) {
                Ok(o) => {
                    debug!(
                        "{}: Patched status {:?} for {}",
                        controlled_by, o.status, o.metadata.name
                    );
                    return Ok(());
                }
                Err(e) => {
                    if let Some(err) = e.api_error() {
                        if err.reason == "Conflict" {
                            warn!(
                                "{}: conflict happen to {}, retry",
                                controlled_by,
                                event.metadata.name.clone()
                            );
                            new_event = config_resource.get(&event.metadata.name)?;
                            continue;
                        }
                    }
                    return Err(e.into());
                }
            }
        }
    }

    /// The workhorse for Instigator.
    /// This will execute only Add, Modify, and Delete phases.
    fn exec(&self, event: OpResource, mut phase: Phase) -> InstigatorResult {
        let name = event.metadata.name.clone();
        let variables = event.spec.variables.clone().unwrap_or_else(|| vec![]);
        let owner_ref = config_owner_reference(name.clone(), event.metadata.uid.clone())?;
        if event.spec.scopes.is_some() {
            let scopes = load_scopes(
                self.client.clone(),
                self.namespace.clone(),
                name.clone(),
                event.spec.clone(),
                variables.clone(),
            )?;
            match phase {
                Phase::Add => {
                    for sc in scopes.iter() {
                        sc.create(owner_ref.clone())?;
                    }
                }
                Phase::Modify => {
                    for sc in scopes.iter() {
                        sc.modify()?;
                    }
                }
                Phase::Delete => {
                    for sc in scopes.iter() {
                        sc.delete()?;
                    }
                }
                _ => {
                    return Err(format_err!(
                        "unknown phase for scopes {:?} on {}",
                        phase.clone(),
                        name.clone(),
                    ))
                }
            }
            // according to the spec, if this is not empty, this AppConfig will be scope instance only, so we could return after we resolved scopes.
            return Ok(());
        }

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
                component.component_name.clone(),
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
            let mut scope_overlap = BTreeMap::new();
            // TODO: if we don't manually add scopes, there are default scopes should be bind
            for sc in &component
                .application_scopes
                .clone()
                .unwrap_or_else(|| vec![])
            {
                let scopes =
                    get_scope_instance(sc.clone(), self.namespace.clone(), self.client.clone())?;
                for scope in scopes.iter() {
                    if !scope.allow_overlap() {
                        if scope_overlap.get(&scope.scope_type()).is_some() {
                            return Err(format_err!(
                                "scope {} {} do not allow overlap",
                                sc.clone(),
                                scope.scope_type()
                            ));
                        }
                        scope_overlap.insert(scope.scope_type(), true);
                    }
                    scope.add(component.clone())?;
                }
            }

            component_updated = true;
            // Resolve variables/parameters

            let parent = get_variable_values(Some(variables.clone()));

            let child = component
                .parameter_values
                .clone()
                .map(|values| resolve_variables(values, variables.clone()))
                .unwrap_or_else(|| Ok(vec![]))?;

            let params = resolve_parameters(
                comp_def.spec.parameters.clone(),
                resolve_values(child, vec![])?,
            )?;

            let inst_name = component.instance_name.clone();
            let new_owner_ref =
                self.get_new_own_ref(phase.clone(), component.clone(), owner_ref.clone())?;

            // Instantiate components
            let workload_meta = self.get_workload_meta(
                name.clone(),
                inst_name.clone(),
                &comp_def,
                &params,
                new_owner_ref.clone(),
                "MainControlLoop".to_string(),
            );
            // Instantiate components
            let workload = self.load_workload_type(&comp_def, workload_meta)?;
            // Load all of the traits related to this component.
            let mut trait_manager = TraitManager {
                config_name: name.clone(),
                instance_name: inst_name.clone(),
                component: component.clone(),
                parent_params: parent.clone(),
                owner_ref: new_owner_ref.clone(),
                workload_type: comp_def.spec.workload_type.clone(),
                traits: vec![], // Always starts empty.
                component_schematic: comp_def.spec.clone(),
            };
            trait_manager.load_traits()?;

            match phase {
                Phase::Add => {
                    info!(
                        "MainControlLoop: Adding component {}",
                        component.component_name.clone()
                    );
                    workload.validate()?;
                    trait_manager.exec(
                        self.namespace.as_str(),
                        self.client.clone(),
                        Phase::PreAdd,
                    )?;
                    workload.add()?;
                    trait_manager.exec(self.namespace.as_str(), self.client.clone(), Phase::Add)?;
                    if let Err(err) = self.event_handler.push_event_message(
                        kube_event::Type::Normal,
                        kube_event::Info {
                            action: "created".to_string(),
                            message: format!(
                                "component {} created",
                                component.component_name.clone(),
                            ),
                            reason: "".to_string(),
                        },
                        get_object_ref(event.clone()),
                    ) {
                        error!("MainControlLoop: adding event err: {:?}", err)
                    }
                }
                Phase::Modify => {
                    info!(
                        "MainControlLoop: Modifying component {}",
                        component.component_name.clone()
                    );

                    workload.validate()?;
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
                    if let Err(err) = self.event_handler.push_event_message(
                        kube_event::Type::Normal,
                        kube_event::Info {
                            action: "updated".to_string(),
                            message: format!(
                                "component {} updated",
                                component.component_name.clone(),
                            ),
                            reason: "".to_string(),
                        },
                        get_object_ref(event.clone()),
                    ) {
                        error!("MainControlLoop: adding event err {:?}", err)
                    }
                }
                Phase::Delete => {
                    info!(
                        "MainControlLoop: Deleting component {}",
                        component.component_name.clone()
                    );
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
                component.component_name.clone(),
                self.client.clone(),
            )?;
            // Resolve variables/parameters
            let parent = get_variable_values(event.spec.variables.clone());
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
                component_schematic: comp_def.spec.clone(),
            };
            trait_manager.load_traits()?;

            info!(
                "MainControlLoop: Deleting component {}",
                component.component_name.clone()
            );
            //The reason for this is that we do not require that traits be deployed only in-cluster.
            //For example, a trait could create an object storage bucket or work with an external API service.
            //So we want to give them a chance to react to a deletion event.
            trait_manager.exec(
                self.namespace.as_str(),
                self.client.clone(),
                Phase::PreDelete,
            )?;
            //delete component instance and let owner_reference to delete real resource
            self.delete_component_instance(component.component_name.clone(), inst_name.clone())?;
            for sc in &component
                .application_scopes
                .clone()
                .unwrap_or_else(|| vec![])
            {
                let scopes =
                    get_scope_instance(sc.clone(), self.namespace.clone(), self.client.clone())?;
                for scope in scopes.iter() {
                    scope.remove(component.clone())?;
                }
            }
            if let Err(err) = self.event_handler.push_event_message(
                kube_event::Type::Normal,
                kube_event::Info {
                    action: "deleted".to_string(),
                    message: format!("component {} deleted", component.component_name.clone(),),
                    reason: "".to_string(),
                },
                get_object_ref(event.clone()),
            ) {
                error!("MainControlLoop: adding event err {:?}", err)
            }
        }
        // if no component was updated or this is an delete phase, just return without status change.
        if !component_updated || phase == Phase::Delete {
            return Ok(());
        }

        let new_record = serde_json::to_string(&new_components)?;
        let mut annotation = event.metadata.annotations.clone();
        annotation.insert(COMPONENT_RECORD_ANNOTATION.to_string(), new_record);
        let default_status = Some(OAMStatus::new(Some(phase.to_string()), None));
        let status = event
            .status
            .clone()
            .map_or(default_status.clone(), |mut hs| {
                hs.phase = Some(phase.to_string());
                Some(hs)
            });

        self.retry_patch_status(
            event,
            status,
            Some(annotation),
            "MainControlLoop".to_string(),
        )
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

    fn get_workload_meta(
        &self,
        config_name: String,
        instance_name: String,
        comp: &KubeComponent,
        params: &ParamMap,
        owner_ref: Option<Vec<meta::OwnerReference>>,
        controlled_by: String,
    ) -> WorkloadMetadata {
        info!(
            "{}: Looking up workload for {} <{}>",
            controlled_by, config_name, comp.metadata.name
        );
        WorkloadMetadata {
            name: config_name,
            instance_name,
            component_name: comp.metadata.name.clone(),
            annotations: Some(comp.metadata.annotations.clone()),
            namespace: self.namespace.clone(),
            definition: comp.spec.clone(),
            client: self.client.clone(),
            params: params.clone(),
            owner_ref,
        }
    }

    fn load_workload_type(
        &self,
        comp: &KubeComponent,
        meta: WorkloadMetadata,
    ) -> Result<Box<dyn WorkloadType>, Error> {
        match comp.spec.workload_type.as_str() {
            workload_type::SERVER_NAME => {
                let rs = ReplicatedServer { meta };
                let workload = CoreWorkloadType::ReplicatedServerType(rs);
                Ok(Box::new(workload))
            }
            workload_type::SINGLETON_SERVER_NAME => {
                let sing = SingletonServer { meta };
                Ok(Box::new(CoreWorkloadType::SingletonServerType(sing)))
            }
            workload_type::SINGLETON_TASK_NAME => {
                let task = SingletonTask { meta };
                Ok(Box::new(CoreWorkloadType::SingletonTaskType(task)))
            }
            workload_type::TASK_NAME => {
                let task = ReplicatedTask {
                    meta,
                    replica_count: Some(1), // Every(1) needs Some(1) to love.
                };
                Ok(Box::new(CoreWorkloadType::ReplicatedTaskType(task)))
            }
            workload_type::SINGLETON_WORKER => {
                let wrkr = SingletonWorker { meta };
                Ok(Box::new(CoreWorkloadType::SingletonWorkerType(wrkr)))
            }
            workload_type::WORKER_NAME => {
                let worker = ReplicatedWorker {
                    meta,
                    replica_count: Some(1), // Every(1) needs Some(1) to love.
                };
                Ok(Box::new(CoreWorkloadType::ReplicatedWorkerType(worker)))
            }
            workload_type::extended_workload::openfaas::OPENFAAS => {
                let openfaas = workload_type::extended_workload::openfaas::OpenFaaS { meta };
                let workload = ExtendedWorkloadType::OpenFaaS(openfaas);
                Ok(Box::new(workload))
            }
            _ => {
                match workload_type::extended_workload::others::Others::new(
                    meta,
                    comp.spec.workload_type.as_str(),
                ) {
                    Err(err) => Err(format_err!(
                        "workloadType {} is unknown, {}",
                        comp.spec.workload_type,
                        err
                    )),
                    Ok(other) => Ok(Box::new(other)),
                }
            }
        }
    }

    fn get_new_own_ref(
        &self,
        phase: Phase,
        component: ComponentConfiguration,
        owner_ref: meta::OwnerReference,
    ) -> Result<Option<Vec<meta::OwnerReference>>, Error> {
        let new = match phase {
            Phase::Add => Some(self.create_component_instance(
                component.component_name.clone(),
                component.instance_name.clone(),
                owner_ref.clone(),
            )?),
            Phase::Modify => {
                let ownref = self.component_instance_owner_reference(
                    component.component_name.clone(),
                    component.instance_name.clone(),
                );
                match ownref {
                    Err(err) => {
                        let e = err.to_string();
                        if !e.contains("NotFound") {
                            // Wrap the error to make it clear where we failed
                            // During deletion, this might indicate that something else
                            // remove the component instance.
                            return Err(format_err!(
                                "{:?} on {}: {}",
                                phase.clone(),
                                component.instance_name.clone(),
                                e
                            ));
                        }
                        Some(self.create_component_instance(
                            component.component_name.clone(),
                            component.instance_name.clone(),
                            owner_ref.clone(),
                        )?)
                    }
                    Ok(own) => Some(own),
                }
            }
            _ => None,
        };
        Ok(new)
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
            "apiVersion": OAM_API_VERSION,
            "kind": "ComponentInstance",
            "metadata": {
                "name": name.clone(),
                "ownerReferences": [{
                    "apiVersion": OAM_API_VERSION,
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
            api_version: OAM_API_VERSION.into(),
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
            api_version: OAM_API_VERSION.into(),
            kind: "ComponentInstance".into(),
            uid: res.metadata.uid.unwrap(),
            controller: Some(true),
            block_owner_deletion: Some(true),
            name: res.metadata.name,
        };
        Ok(vec![owner])
    }

    fn component_instance_set_status(
        &self,
        component_name: String,
        instance_name: String,
        status: String,
    ) -> Result<(), Error> {
        let name = combine_name(component_name, instance_name);
        let crd_req = RawApi::customResource("componentinstances")
            .group(CONFIG_GROUP)
            .version(CONFIG_VERSION)
            .within(self.namespace.as_str());
        let req = crd_req.get(name.as_str())?;
        let mut res: KubeComponentInstance = self.client.request(req)?;
        res.status = Some(status);
        let req = crd_req.patch(
            name.as_str(),
            &PatchParams::default(),
            serde_json::to_vec(&res)?,
        )?;
        let _: KubeComponentInstance = self.client.request(req)?;
        Ok(())
    }
}

pub fn get_object_ref(event: OpResource) -> ObjectReference {
    ObjectReference {
        api_version: event.types.apiVersion.clone(),
        kind: event.types.kind.clone(),
        name: Some(event.metadata.name.clone()),
        field_path: None,
        namespace: event.metadata.namespace.clone(),
        resource_version: event.metadata.resourceVersion.clone(),
        uid: event.metadata.uid.clone(),
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
                api_version: OAM_API_VERSION.into(),
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
        .group("core.oam.dev")
        .within(&namespace);
    let comp_def_req = component_resource.get(comp_name.as_str())?;
    let comp_def: KubeComponent = match client.request::<KubeComponent>(comp_def_req) {
        Ok(comp) => comp,
        Err(err) => {
            return Err(format_err!(
                "get component {} err: {}",
                comp_name.as_str(),
                err
            ))
        }
    };
    Ok(comp_def)
}

pub fn get_values(values: Option<Vec<ParameterValue>>) -> Vec<ParameterValue> {
    values.or_else(|| Some(vec![])).unwrap()
}

/// Load application scopes from scope_bindings
/// check if there is not allowed overlap
pub fn load_scopes(
    client: APIClient,
    namespace: String,
    instance_name: String,
    spec: ApplicationConfiguration,
    variables: Vec<Variable>,
) -> Result<Vec<OAMScope>, failure::Error> {
    let mut scopes: Vec<OAMScope> = vec![];
    for scope_binding in spec.scopes.iter() {
        for sc in scope_binding.iter() {
            let param = sc
                .parameter_values
                .clone()
                .map(|values| resolve_variables(values, variables.clone()))
                .unwrap_or_else(|| Ok(vec![]))
                .unwrap();
            debug!(
                "param {:?} extracted from properties {:?} and variables {:?}",
                param.clone(),
                sc.parameter_values.clone(),
                variables.clone()
            );
            let scope = load_scope(
                client.clone(),
                namespace.clone(),
                instance_name.clone(),
                &sc,
                param.clone(),
            )?;
            scopes.insert(scopes.len(), scope);
        }
    }
    Ok(scopes)
}

// load_scope should load scope from k8s crd
// NOTE: this is a temporary solution just return core scope here
fn load_scope(
    client: APIClient,
    namespace: String,
    instance_name: String,
    binding: &ScopeBinding,
    param: Vec<ParameterValue>,
) -> Result<OAMScope, failure::Error> {
    debug!("Scope binding params: {:?}", &binding.parameter_values);
    load_scope_by_type(
        client.clone(),
        namespace,
        instance_name,
        binding.scope_type.as_str(),
        param,
    )
}

fn load_scope_by_type(
    client: APIClient,
    namespace: String,
    instance_name: String,
    scope_type: &str,
    param: Vec<ParameterValue>,
) -> Result<OAMScope, failure::Error> {
    match scope_type {
        scopes::NETWORK_SCOPE => Ok(OAMScope::Network(Network::from_params(
            instance_name.clone(),
            namespace.clone(),
            client.clone(),
            param,
        )?)),
        scopes::HEALTH_SCOPE => Ok(OAMScope::Health(Health::from_params(
            instance_name.clone(),
            namespace.clone(),
            client.clone(),
            param,
        )?)),
        _ => Err(format_err!(
            "unknown scope {} type {}",
            instance_name,
            scope_type
        )),
    }
}

type KubeOpsConfig = Object<ApplicationConfiguration, OAMStatus>;

//get_scope_instance load scope instance by load AppConfig object
fn get_scope_instance(
    name: String,
    ns: String,
    client: APIClient,
) -> Result<Vec<OAMScope>, failure::Error> {
    let mut scopes = vec![];
    let resource = RawApi::customResource(CONFIG_CRD)
        .within(ns.as_str())
        .group(CONFIG_GROUP)
        .version(CONFIG_VERSION);
    //init all the existing objects at initiate, this should be done by informer
    let req = resource.get(name.as_str())?;
    let cfg = client.request::<KubeOpsConfig>(req)?;
    for scope_binding in cfg.spec.scopes.clone().unwrap_or_else(|| vec![]).iter() {
        let param = scope_binding
            .parameter_values
            .clone()
            .map(|values| {
                resolve_variables(values, cfg.spec.variables.clone().unwrap_or_else(|| vec![]))
            })
            .unwrap_or_else(|| Ok(vec![]))
            .unwrap();
        let scope = load_scope(
            client.clone(),
            ns.clone(),
            cfg.metadata.name.clone(),
            scope_binding,
            param,
        )?;
        scopes.insert(scopes.len(), scope)
    }
    Ok(scopes)
}
