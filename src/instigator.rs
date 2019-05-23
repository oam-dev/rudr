use kube::{
    api::Resource,
    api::Reflector,
    client::APIClient,
};
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;

use crate::schematic::{
    configuration::Configuration,
    component::Component,
    Status,
};

/// An Instigator takes an inbound object and manages the reconcilliation with the desired objects.
/// 
/// Instigators know how to deal with the following operations:
/// - Add
/// - Modify
/// - Delete
#[derive(Clone)]
pub struct Instigator {
    client: APIClient,
    cache: Reflector<Component, Status>
}

pub type InstigatorResult = Result<(), failure::Error>;



impl Instigator {
    pub fn new(client: kube::client::APIClient, cache: Reflector<Component, Status>) -> Self {
        Instigator {
            client: client,
            cache: cache,
        }
    }
    pub fn add(&self, event: Resource<Configuration, Status>) -> InstigatorResult {
        // Find components
        let cache = self.cache.read().unwrap();
        let comp_def = cache.get(event.spec.component.as_str());

        if comp_def.is_none() {
            return Err(format_err!("Component {} not found", event.spec.component))
        }
        println!("Found component {}", event.spec.component);

        // Resolve parameters
        // Instantiate components
        self.instantiate_components(event.metadata.name, comp_def.unwrap())?;
        // Attach traits
        Ok(())
    }
    pub fn modify(&self) -> InstigatorResult {
        Ok(())
    }
    pub fn delete(&self) -> InstigatorResult {
        Ok(())
    }

    fn instantiate_components(&self, name: String, comp: &Resource<Component, Status>) -> InstigatorResult {
        println!("Adding {}", name);
        match comp.spec.workload_type.as_str() {
            "core.hydra.io/v1alpha1.Singleton" => {
                let single = Singleton::new(name, "default".into(), comp.spec.clone());
                single.add(self.client.clone())
            },
            //"core.hydra.io/v1alpha1.ReplicableService" => {},
            //"core.hydra.io/v1alpha1.Task" => {},
            //"core.hydra.io/v1alpha1.ReplicableTask" => {},
            _ => {
                Err(format_err!("workloadType {} is unknown", comp.spec.workload_type))
            }
        }
    }
}

#[derive(Debug)]
struct Singleton {
    name: String,
    namespace: String,
    definition: Component,
    //client: APIClient,
}
impl Singleton {
    fn new(name: String, namespace: String, definition: Component) -> Self {
        Singleton {
            name: name,
            namespace: namespace,
            definition: definition,
        }
    }
    fn add(&self, client: APIClient) -> InstigatorResult {
        let pod = self.to_pod();
        let (req, _) = api::Pod::create_namespaced_pod(self.namespace.as_str(), &pod, Default::default())?;
        let res = client.request(req);
        println!("{:?}", res);
        res
    }
    fn to_pod(&self) -> api::Pod {
        api::Pod{
            // TODO: Could make this generic.
            metadata: Some(meta::ObjectMeta{
                name: Some(self.name.clone()),
                ..Default::default()
            }),
            spec: Some(self.definition.to_pod_spec()),
            ..Default::default()
        }
    }
}

