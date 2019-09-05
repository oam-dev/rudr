use k8s_openapi::api::apps::v1 as apps;
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::api::{PatchParams, PostParams};

use crate::workload_type::workload_builder::ServiceBuilder;
use crate::workload_type::{InstigatorResult, KubeName, WorkloadMetadata, WorkloadType};

use std::collections::BTreeMap;
use std::{thread, time};

/// A Replicated Service can take one component and scale it up or down.
pub struct ReplicatedService {
    pub meta: WorkloadMetadata,
}

impl ReplicatedService {
    /// Create a Pod definition that describes this Singleton
    fn to_deployment(&self) -> apps::Deployment {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert(
            "workload-type".to_string(),
            "replicated-service".to_string(),
        );

        apps::Deployment {
            // TODO: Could make this generic.
            metadata: Some(meta::ObjectMeta {
                name: Some(self.kube_name()),
                labels: Some(labels),
                owner_references: self.meta.owner_ref.clone(),
                ..Default::default()
            }),
            spec: Some(
                self.meta
                    .definition
                    .to_deployment_spec(self.meta.name.clone(), self.meta.params.clone()),
            ),
            ..Default::default()
        }
    }
}

impl KubeName for ReplicatedService {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}

impl WorkloadType for ReplicatedService {
    fn add(&self) -> InstigatorResult {
        let deployment = self.to_deployment();
        let deployments = kube::api::Api::v1Deployment(self.meta.client.clone())
            .within(self.meta.namespace.as_str());
        let pp = PostParams::default();
        deployments.create(&pp, serde_json::to_vec(&deployment)?)?;

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "service".to_string());
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(labels)
            .owner_reference(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")
    }
    fn modify(&self) -> InstigatorResult {
        let deployment = self.to_deployment();
        let deployments = kube::api::Api::v1Deployment(self.meta.client.clone())
            .within(self.meta.namespace.as_str());
        let pp = PatchParams::default();
        deployments.patch(
            self.kube_name().as_str(),
            &pp,
            serde_json::to_vec(&deployment)?,
        )?;

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "service".to_string());
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(labels)
            .owner_reference(self.meta.owner_ref.clone())
            .do_request(
                self.meta.client.clone(),
                self.meta.namespace.clone(),
                "modify",
            )
    }
}

/// Singleton represents the Singleton Workload Type, as defined in the Hydra specification.
///
/// It is currently implemented as a Kubernetes Pod with a Service in front of it.
pub struct SingletonService {
    pub meta: WorkloadMetadata,
}
impl SingletonService {
    /// Create a Pod definition that describes this Singleton
    fn to_pod(&self) -> api::Pod {
        let mut labels = BTreeMap::new();
        let podname = self.kube_name();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "singleton-service".to_string());
        api::Pod {
            // TODO: Could make this generic.
            metadata: Some(meta::ObjectMeta {
                name: Some(podname),
                labels: Some(labels),
                owner_references: self.meta.owner_ref.clone(),
                ..Default::default()
            }),
            spec: Some(self.meta.definition.to_pod_spec(self.meta.params.clone())),
            ..Default::default()
        }
    }
}

impl KubeName for SingletonService {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}
impl WorkloadType for SingletonService {
    fn add(&self) -> InstigatorResult {
        let pod = self.to_pod();
        let pp = kube::api::PostParams::default();
        kube::api::Api::v1Pod(self.meta.client.clone())
            .within(self.meta.namespace.as_str())
            .create(&pp, serde_json::to_vec(&pod)?)?;
        // Create service
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "singleton-service".to_string());
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(labels)
            .owner_reference(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")
    }

    fn modify(&self) -> InstigatorResult {
        //because pod upgrade have many restrictions, so we delete and create a new one to consistent with components.
        let dp = kube::api::DeleteParams::default();
        if let Err(err) = kube::api::Api::v1Pod(self.meta.client.clone())
            .within(self.meta.namespace.as_str())
            .delete(self.kube_name().as_str(), &dp)
        {
            match err.kind() {
                kube::ErrorKind::Api(e) => {
                    if e.code != 404 {
                        return Err(failure::err_msg(err.to_string()));
                    }
                }
                _ => return Err(failure::err_msg(err.to_string())),
            }
        }
        //FIXME: Here we should check pod's terminating status until it's deleted completely.
        thread::sleep(time::Duration::from_secs(5));
        //create new one
        let pod = self.to_pod();
        let pp = kube::api::PostParams::default();
        kube::api::Api::v1Pod(self.meta.client.clone())
            .within(self.meta.namespace.as_str())
            .create(&pp, serde_json::to_vec(&pod)?)?;
        // Update service
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "singleton-service".to_string());
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(labels)
            .owner_reference(self.meta.owner_ref.clone())
            .do_request(
                self.meta.client.clone(),
                self.meta.namespace.clone(),
                "modify",
            )
    }
}

#[cfg(test)]
mod test {
    use kube::{client::APIClient, config::Configuration};

    use crate::schematic::component::Component;
    use crate::workload_type::{service::*, KubeName, WorkloadMetadata};

    use std::collections::BTreeMap;

    #[test]
    fn test_singleton_service_kube_name() {
        let cli = APIClient::new(mock_kube_config());

        let sing = SingletonService {
            meta: WorkloadMetadata {
                name: "de".into(),
                component_name: "hydrate".into(),
                instance_name: "squidgy".into(),
                namespace: "tests".into(),
                definition: Component {
                    ..Default::default()
                },
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
        };

        assert_eq!("squidgy", sing.kube_name().as_str());
    }

    #[test]
    fn test_replicated_service_kube_name() {
        let cli = APIClient::new(mock_kube_config());

        let rs = ReplicatedService {
            meta: WorkloadMetadata {
                name: "de".into(),
                component_name: "hydrate".into(),
                instance_name: "dehydrate".into(),
                namespace: "tests".into(),
                definition: Component {
                    ..Default::default()
                },
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
        };

        assert_eq!("dehydrate", rs.kube_name().as_str());
    }

    /// This mock builds a KubeConfig that will not be able to make any requests.
    fn mock_kube_config() -> Configuration {
        Configuration {
            base_path: ".".into(),
            client: reqwest::Client::new(),
        }
    }

}
