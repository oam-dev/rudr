use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

use crate::schematic::component::Component;
use crate::workload_type::{InstigatorResult, KubeName, ParamMap, WorkloadType};

use serde_json::to_string_pretty as to_json;
use std::collections::BTreeMap;

/// Singleton represents the Singleton Workload Type, as defined in the Hydra specification.
///
/// It is currently implemented as a Kubernetes Pod with a Service in front of it.
pub struct Singleton {
    pub name: String,
    pub component_name: String,
    pub instance_name: String,
    pub namespace: String,
    pub definition: Component,
    pub client: APIClient,
    pub params: ParamMap,
    pub owner_ref: Option<Vec<meta::OwnerReference>>,
}
impl Singleton {
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
                owner_references: self.owner_ref.clone(),
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
                    owner_references: self.owner_ref.clone(),
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

impl KubeName for Singleton {
    fn kube_name(&self) -> String {
        self.instance_name.to_string()
    }
}
impl WorkloadType for Singleton {
    fn add(&self) -> InstigatorResult {
        let pod = self.to_pod();
        let pp = kube::api::PostParams::default();
        kube::api::Api::v1Pod(self.client.clone())
            .within(self.namespace.as_str())
            .create(&pp, serde_json::to_vec(&pod)?)?;
        match self.to_service() {
            Some(svc) => {
                info!("Service:\n{}", to_json(&svc).unwrap());
                let pp = kube::api::PostParams::default();
                kube::api::Api::v1Service(self.client.clone())
                    .within(self.namespace.as_str())
                    .create(&pp, serde_json::to_vec(&svc)?)?;
                Ok(())
            }
            // No service to create
            None => {
                info!("Not attaching service to pod with no container ports.");
                Ok(())
            }
        }
    }
}
