use k8s_openapi::api::apps::v1 as apps;
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

use crate::schematic::component::Component;

use serde_json::to_string_pretty as to_json;
use std::collections::BTreeMap;

/// The fully qualified name of a replicated service.
pub const REPLICATED_SERVICE_NAME: &'static str = "core.hydra.io/v1alpha1.ReplicatedService";
/// The fully qualified name of a singleton.
pub const SINGLETON_NAME: &'static str = "core.hydra.io/v1alpha1.Singleton";

type InstigatorResult = Result<(), failure::Error>;
type ParamMap = BTreeMap<String, serde_json::Value>;

/// WorkloadType describes one of the available workload types.
///
/// An implementation of a workload type must be able to add, modify, and delete itself.
pub trait WorkloadType {
    fn add(&self) -> InstigatorResult;
    fn modify(&self) -> InstigatorResult;
    fn delete(&self) -> InstigatorResult;
}

pub enum CoreWorkloadType {
    SingletonType(Singleton),
    ReplicatedServiceType(ReplicatedService),
}

impl CoreWorkloadType {
    pub fn delete(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonType(sing) => sing.delete(),
            CoreWorkloadType::ReplicatedServiceType(repl) => repl.delete(),
        }
    }
    pub fn add(&self) -> InstigatorResult {
        match self {
            CoreWorkloadType::SingletonType(sing) => sing.add(),
            CoreWorkloadType::ReplicatedServiceType(repl) => repl.add(),
        }
    }
    pub fn modify(&self) -> InstigatorResult {
        Err(format_err!("modify operation is not implemented"))
    }
}

/// Singleton represents the Singleton Workload Type, as defined in the Hydra specification.
///
/// It is currently implemented as a Kubernetes Pod with a Service in front of it.
pub struct Singleton {
    pub name: String,
    pub component_name: String,
    pub namespace: String,
    pub definition: Component,
    pub client: APIClient,
    pub params: ParamMap,
}
impl Singleton {
    fn kube_name(&self) -> String {
        format!("{}-{}", self.name.as_str(), self.component_name.as_str())
    }
    /// Create a Pod definition that describes this Singleton
    fn to_pod(&self) -> api::Pod {
        let mut labels = BTreeMap::new();
        let podname = self.kube_name();
        labels.insert("app".to_string(), self.name.clone());
        api::Pod {
            // TODO: Could make this generic.
            metadata: Some(meta::ObjectMeta {
                name: Some(podname),
                labels: Some(labels),
                ..Default::default()
            }),
            spec: Some(self.definition.to_pod_spec()),
            ..Default::default()
        }
    }
    /// Create a service if this component has a port.
    fn to_service(&self) -> Option<api::Service> {
        self.definition.listening_port().and_then(|port| {
            let mut labels = BTreeMap::new();
            labels.insert("app".to_string(), self.name.clone());
            Some(api::Service {
                metadata: Some(meta::ObjectMeta {
                    name: Some(self.kube_name()),
                    labels: Some(labels.clone()),
                    ..Default::default()
                }),
                spec: Some(api::ServiceSpec {
                    selector: Some(labels),
                    ports: Some(vec![port.to_service_port()]),
                    ..Default::default()
                }),
                ..Default::default()
            })
        })
    }
}
impl WorkloadType for Singleton {
    fn add(&self) -> InstigatorResult {
        let pod = self.to_pod();
        let (req, _) =
            api::Pod::create_namespaced_pod(self.namespace.as_str(), &pod, Default::default())?;

        // We force the decoded value into a serde_json::Value because we don't care if Kubernetes returns a
        // malformed body. We just want the response code validated by APIClient.
        let res: Result<serde_json::Value, failure::Error> = self.client.request(req);
        if res.is_err() {
            return Err(res.unwrap_err());
        }

        /*
        if res.is_err() {
            // FIXME: We seem to be getting a formatting error for the response, but I don't know what is causing it
            println!("Pod: {}", to_json(&pod).unwrap());
            println!("Warning: {:?}", res);
        }
        */

        match self.to_service() {
            Some(svc) => {
                println!("Service:\n{}", to_json(&svc).unwrap());
                let (sreq, _) = api::Service::create_namespaced_service(
                    self.namespace.as_str(),
                    &svc,
                    Default::default(),
                )?;
                let sres: Result<serde_json::Value, failure::Error> = self.client.request(sreq);
                sres.and_then(|_o| Ok(()))
            }
            // No service to create
            None => {
                println!("Not attaching service to pod with no container ports.");
                Ok(())
            }
        }
    }
    fn modify(&self) -> InstigatorResult {
        Err(format_err!("Not implemented"))
    }
    fn delete(&self) -> InstigatorResult {
        let (req, _) = api::Pod::delete_namespaced_pod(
            self.name.as_str(),
            self.namespace.as_str(),
            Default::default(),
        )?;

        // By decoding into serde_json::Value, we are bypassing all checks on the return data, which
        // is fine. We don't actually need any of it, and the APIClient checks for status and error
        // on our behalf.
        let pres: Result<serde_json::Value, failure::Error> = self.client.request(req);

        match self.to_service() {
            Some(_) => {
                let (sreq, _) = api::Service::delete_namespaced_service(
                    self.name.as_str(),
                    self.namespace.as_str(),
                    Default::default(),
                )?;
                let sres: Result<serde_json::Value, failure::Error> = self.client.request(sreq);
                // If either op fails, return the error. Otherwise, just return Ok(()).
                pres.and(sres).and_then(|_o| Ok(()))
            }
            None => pres.and_then(|_| Ok(())),
        }
    }
}

/// A Replicated Service can take one component and scale it up or down.
pub struct ReplicatedService {
    pub name: String,
    pub namespace: String,
    pub component_name: String,
    pub definition: Component,
    pub client: APIClient,
    pub params: ParamMap,
}

impl ReplicatedService {
    /// Create a Pod definition that describes this Singleton
    fn to_deployment(&self) -> apps::Deployment {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.name.clone());
        apps::Deployment {
            // TODO: Could make this generic.
            metadata: Some(meta::ObjectMeta {
                name: Some(self.kube_name()),
                labels: Some(labels),
                ..Default::default()
            }),
            spec: Some(self.definition.to_deployment_spec(self.name.clone())),
            ..Default::default()
        }
    }
    /// Create a service if this component has a port.
    fn to_service(&self) -> Option<api::Service> {
        self.definition.listening_port().and_then(|port| {
            let mut labels = BTreeMap::new();
            labels.insert("app".to_string(), self.name.clone());
            Some(api::Service {
                metadata: Some(meta::ObjectMeta {
                    name: Some(self.kube_name()),
                    labels: Some(labels.clone()),
                    ..Default::default()
                }),
                spec: Some(api::ServiceSpec {
                    selector: Some(labels),
                    ports: Some(vec![port.to_service_port()]),
                    ..Default::default()
                }),
                ..Default::default()
            })
        })
    }
    fn kube_name(&self) -> String {
        format!("{}-{}", self.name.as_str(), self.component_name.as_str())
    }
}

impl WorkloadType for ReplicatedService {
    fn add(&self) -> InstigatorResult {
        let deployment = self.to_deployment();
        let (req, _) = apps::Deployment::create_namespaced_deployment(
            self.namespace.as_str(),
            &deployment,
            Default::default(),
        )?;

        // We force the decoded value into a serde_json::Value because we don't care if Kubernetes returns a
        // malformed body. We just want the response code validated by APIClient.
        let res: Result<serde_json::Value, failure::Error> = self.client.request(req);
        if res.is_err() {
            return Err(res.unwrap_err());
        }
        match self.to_service() {
            Some(svc) => {
                println!("Service:\n{}", to_json(&svc).unwrap());
                let (sreq, _) = api::Service::create_namespaced_service(
                    self.namespace.as_str(),
                    &svc,
                    Default::default(),
                )?;
                let sres: Result<serde_json::Value, failure::Error> = self.client.request(sreq);
                res.and(sres).and_then(|_o| Ok(()))
            }
            // No service to create
            None => {
                println!("Not attaching service to pod with no container ports.");
                Ok(())
            }
        }
    }
    fn modify(&self) -> InstigatorResult {
        Err(format_err!("Not implemented"))
    }
    fn delete(&self) -> InstigatorResult {
        let (req, _) = apps::Deployment::delete_namespaced_deployment(
            self.name.as_str(),
            self.namespace.as_str(),
            Default::default(),
        )?;

        let dres: Result<serde_json::Value, failure::Error> = self.client.request(req);

        match self.to_service() {
            Some(_) => {
                let (sreq, _) = api::Service::delete_namespaced_service(
                    self.name.as_str(),
                    self.namespace.as_str(),
                    Default::default(),
                )?;
                let sres: Result<serde_json::Value, failure::Error> = self.client.request(sreq);
                sres.and_then(|_| Ok(()))
            }
            None => dres.and_then(|_| Ok(())),
        }
    }
}
