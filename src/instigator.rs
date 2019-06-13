use kube::{api::Reflector, api::Resource, client::APIClient};
use std::collections::BTreeMap;

use crate::{
    schematic::{
        component::Component,
        configuration::OperationalConfiguration,
        parameter::{resolve_parameters, resolve_values},
        traits::{Ingress, TraitBinding, TraitImplementation},
        Status,
    },
    workload_type::{CoreWorkloadType, ReplicatedService, Singleton},
};

/// Type alias for the results that all instantiation operations return
pub type InstigatorResult = Result<(), failure::Error>;

/// This error is returned when a component cannot be found.
#[derive(Fail, Debug)]
#[fail(display = "Component {} not found", name)]
pub struct ComponentNotFoundError {
    name: String,
}

const DEFAULT_NAMESPACE: &'static str = "default";

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
    cache: Reflector<Component, Status>,
}

type OpResource = Resource<OperationalConfiguration, Status>;
type ParamMap = BTreeMap<String, serde_json::Value>;

// The implementation of Instegator can probably be cleaned up quite a bit.
// My bad Go habits of recklessly duplicating code may not be justified here.

impl Instigator {
    /// Create a new instigator
    ///
    /// An instigator uses the reflector as a cache of Components, and will use the API client
    /// for creating and managing the component implementation.
    pub fn new(client: kube::client::APIClient, cache: Reflector<Component, Status>) -> Self {
        Instigator {
            client: client,
            cache: cache,
        }
    }

    /// Create new Kubernetes objects based on this config.
    pub fn add(&self, event: OpResource) -> InstigatorResult {
        let name = event.metadata.name.clone();

        // component cache
        let cache = self.cache.read().unwrap();

        // TODO:

        // - Resolve scope bindings

        for component in event.spec.components.unwrap_or(vec![]) {
            let comp_def = cache
                .get(component.name.as_str())
                .ok_or(ComponentNotFoundError {
                    name: component.name.clone(),
                })?;

            // - Resolve parameters
            let parent = event
                .spec
                .parameter_values
                .clone()
                .or(Some(vec![]))
                .unwrap();
            let child = component.parameter_values.clone().or(Some(vec![])).unwrap();
            let merged_vals = resolve_values(child, parent)?;
            let params = resolve_parameters(comp_def.spec.parameters.clone(), merged_vals)?;

            // Instantiate components
            let workload = self.load_workload_type(name.clone(), comp_def, &params)?;
            println!("Adding component {}", component.name.clone());
            workload.add()?;
            // Attach traits
            // FIXME: This is currently not working because workload.add is returning an error having to do with the
            // formatting of the response object. :angry-eyes:
            for t in component.traits.unwrap_or(vec![]).iter() {
                println!("Searching for trait {}", t.name.as_str());
                let cname = component.name.clone();
                let imp = self.load_trait(name.clone(), cname, t)?;
                imp.add(DEFAULT_NAMESPACE.into(), self.client.clone())?;
            }
        }
        Ok(())
    }
    /// Modify existing Kubernetes objects based on config and workload type.
    pub fn modify(&self, event: OpResource) -> InstigatorResult {
        // Find components
        let cache = self.cache.read().unwrap();

        // TODO:
        // Resolve parameters
        // Resolve scopes

        // Modify components
        for component in event.spec.components.unwrap_or(vec![]) {
            let cname = component.name.clone();
            let comp_def = cache.get(cname.as_str()).ok_or(ComponentNotFoundError {
                name: component.name,
            })?;

            // - Resolve parameters
            let parent = event
                .spec
                .parameter_values
                .clone()
                .or(Some(vec![]))
                .unwrap();
            let child = component.parameter_values.clone().or(Some(vec![])).unwrap();
            let merged_vals = resolve_values(child, parent)?;
            let params = resolve_parameters(comp_def.spec.parameters.clone(), merged_vals)?;

            let workload =
                self.load_workload_type(event.metadata.name.clone(), comp_def, &params)?;
            workload.modify()?;
        }
        Ok(())
    }
    /// Delete the Kubernetes objects associated with this config.
    pub fn delete(&self, event: OpResource) -> InstigatorResult {
        let name = event.metadata.name.clone();
        let cache = self.cache.read().unwrap();

        // TODO:
        // Resolve params
        // Delete from scopes

        for component in event.spec.components.unwrap_or(vec![]) {
            // Find component
            let cname = component.name.clone();
            let comp_def = cache.get(cname.as_str()).ok_or(ComponentNotFoundError {
                name: cname.clone(),
            })?;

            // - Resolve parameters
            let parent = event
                .spec
                .parameter_values
                .clone()
                .or(Some(vec![]))
                .unwrap();
            let child = component.parameter_values.clone().or(Some(vec![])).unwrap();
            let merged_vals = resolve_values(child, parent)?;
            let params = resolve_parameters(comp_def.spec.parameters.clone(), merged_vals)?;

            // Delete traits
            // Right now, a failed deletion on a trait is just logged, and is not
            // a fatail error.
            for t in component.traits.unwrap_or(vec![]).iter() {
                println!("Deleting trait {}", t.name.as_str());
                let imp = self.load_trait(name.clone(), cname.clone(), t)?;
                let res = imp.delete(DEFAULT_NAMESPACE.into(), self.client.clone());
                if res.is_err() {
                    println!(
                        "Error deleting trait for {}: {}",
                        t.name.as_str(),
                        res.unwrap_err()
                    );
                }
            }

            // Delete component
            self.load_workload_type(event.metadata.name.clone(), comp_def, &params)?
                .delete()?;
        }
        Ok(())
    }

    fn load_workload_type(
        &self,
        name: String,
        comp: &Resource<Component, Status>,
        params: &ParamMap,
    ) -> Result<CoreWorkloadType, failure::Error> {
        println!("Looking up {}", name);
        match comp.spec.workload_type.as_str() {
            "core.hydra.io/v1alpha1.ReplicatedService" => {
                let rs = ReplicatedService {
                    name: name,
                    component_name: comp.metadata.name.clone(),
                    namespace: DEFAULT_NAMESPACE.into(),
                    definition: comp.spec.clone(),
                    client: self.client.clone(),
                    params: params.clone(),
                };
                Ok(CoreWorkloadType::ReplicatedServiceType(rs))
            }
            "core.hydra.io/v1alpha1.Singleton" => {
                let sing = Singleton {
                    name: name,
                    namespace: DEFAULT_NAMESPACE.into(),
                    component_name: comp.metadata.name.clone(),
                    definition: comp.spec.clone(),
                    client: self.client.clone(),
                    params: params.clone(),
                };
                Ok(CoreWorkloadType::SingletonType(sing))
            }
            //"core.hydra.io/v1alpha1.Task" => {},
            //"core.hydra.io/v1alpha1.ReplicableTask" => {},
            _ => Err(format_err!(
                "workloadType {} is unknown",
                comp.spec.workload_type
            )),
        }
    }

    fn load_trait(
        &self,
        name: String,
        component_name: String,
        binding: &TraitBinding,
    ) -> Result<impl TraitImplementation, failure::Error> {
        match binding.name.as_str() {
            "ingress" => Ok(Ingress::new(80, name, component_name, None, None)),
            _ => Err(format_err!("unknown trait {}", binding.name)),
        }
    }
}
