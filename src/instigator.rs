use kube::{
    api::Resource,
    client::APIClient,
};
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;

use crate::schematic::{
    Component,
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
}

pub type InstigatorResult = Result<(), failure::Error>;



impl Instigator {
    pub fn new(client: kube::client::APIClient) -> Self {
        Instigator {
            client: client,
        }
    }
    pub fn add(&self, event: Resource<Component, Status>) -> InstigatorResult {
        println!("Adding {}", event.metadata.name);
        match event.spec.workload_type.as_str() {
            "core.hydra.io/v1alpha1.Singleton" => {
                let single = Singleton::new(event.metadata.name, "default".into(), event.spec.clone());
                return single.add(self.client.clone())
            },
            "core.hydra.io/v1alpha1.ReplicableService" => {},
            "core.hydra.io/v1alpha1.Task" => {},
            "core.hydra.io/v1alpha1.ReplicableTask" => {},
            _ => {
                return Err(format_err!("workloadType {} is unknown", event.spec.workload_type));
            }
        }
        Ok(())
    }
    pub fn modify(&self) -> InstigatorResult {
        Ok(())
    }
    pub fn delete(&self) -> InstigatorResult {
        Ok(())
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
        let containers = self.definition.containers.iter().map( |c| {
            api::Container {
                image: Some(c.image.clone()),
                name: c.name.clone(),
                ..Default::default()
            }
        }).collect();
        
        let pod = api::Pod{
            metadata: Some(meta::ObjectMeta{
                name: Some(self.name.clone()),
                ..Default::default()
            }),
            spec: Some(api::PodSpec {
                containers: containers,
                ..Default::default()                
            }),
            ..Default::default()
        };

        let (req, _) = api::Pod::create_namespaced_pod(self.namespace.as_str(), &pod, Default::default())?;
        let res = client.request(req);
        println!("{:?}", res);
        res
    }
}

