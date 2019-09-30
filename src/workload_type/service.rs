use k8s_openapi::api::apps::v1 as apps;
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::api::Object;

use crate::workload_type::workload_builder::ServiceBuilder;
use crate::workload_type::{
    InstigatorResult, KubeName, StatusResult, WorkloadMetadata, WorkloadType,
};

use std::collections::BTreeMap;

/// A Replicated Service can take one component and scale it up or down.
pub struct ReplicatedService {
    pub meta: WorkloadMetadata,
}

impl ReplicatedService {
    fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "Service".to_string());
        labels
    }
}

pub fn to_config_maps(
    configs: BTreeMap<String, BTreeMap<String, String>>,
    owner_ref: Option<Vec<meta::OwnerReference>>,
    labels: Option<BTreeMap<String, String>>,
) -> Vec<api::ConfigMap> {
    let mut new_configs: Vec<api::ConfigMap> = vec![];
    for (key, values) in configs {
        new_configs.insert(
            new_configs.len(),
            api::ConfigMap {
                metadata: Some(meta::ObjectMeta {
                    owner_references: owner_ref.clone(),
                    labels: labels.clone(),
                    name: Some(key),
                    ..Default::default()
                }),
                data: Some(values),
                ..Default::default()
            },
        );
    }
    new_configs
}

impl KubeName for ReplicatedService {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}

impl WorkloadType for ReplicatedService {
    fn add(&self) -> InstigatorResult {
        //pre create config_map
        self.meta.create_config_maps("Service")?;
        self.meta.create_deployment("Service")?;

        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
            .select_labels(self.meta.select_labels())
            .owner_reference(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")
    }
    fn modify(&self) -> InstigatorResult {
        //TODO update config_map
        self.meta.update_deployment("Service")?;

        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
            .select_labels(self.meta.select_labels())
            .owner_reference(self.meta.owner_ref.clone())
            .do_request(
                self.meta.client.clone(),
                self.meta.namespace.clone(),
                "modify",
            )
    }
    fn delete(&self) -> InstigatorResult {
        self.meta.delete_deployment()?;
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
        )
    }
    fn status(&self) -> StatusResult {
        let deploy: Object<_, apps::DeploymentStatus> =
            match kube::api::Api::v1Deployment(self.meta.client.clone())
                .within(self.meta.namespace.as_str())
                .get_status(self.kube_name().as_str())
            {
                Ok(deploy) => deploy,
                Err(e) => return Ok(e.to_string()),
            };

        let status: apps::DeploymentStatus = deploy.status.unwrap();
        let replica = status.replicas.unwrap_or(0);
        let available_replicas = status.available_replicas.unwrap_or(0);
        let unavailable_replicas = status.unavailable_replicas.unwrap_or(0);

        if available_replicas == replica {
            return Ok("running".to_string());
        }
        if unavailable_replicas > 0 {
            return Ok("unavailable".to_string());
        }
        Ok("updating".to_string())
    }
}

/// Singleton represents the Singleton Workload Type, as defined in the Hydra specification.
///
/// It is currently implemented as a Kubernetes Pod with a Service in front of it.
pub struct SingletonService {
    pub meta: WorkloadMetadata,
}
impl SingletonService {
    fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "SingletonService".to_string());
        labels
    }
    /// Create a Pod definition that describes this Singleton
    fn to_pod(&self) -> api::Pod {
        let podname = self.kube_name();
        api::Pod {
            metadata: Some(meta::ObjectMeta {
                annotations: self.meta.annotations.clone(),
                name: Some(podname),
                labels: Some(self.labels()),
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
        //pre create config_map
        self.meta.create_config_maps("singleton-service")?;

        let pod = self.to_pod();
        let pp = kube::api::PostParams::default();
        kube::api::Api::v1Pod(self.meta.client.clone())
            .within(self.meta.namespace.as_str())
            .create(&pp, serde_json::to_vec(&pod)?)?;
        // Create service
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
            .select_labels(self.meta.select_labels())
            .owner_reference(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")
    }

    //TODO: because pod upgrade have many restrictions and very complicated, so we don't support now.
    //User should delete and create a new SingletonService to solve this.
    fn modify(&self) -> InstigatorResult {
        Err(format_err!(
            "we don't support SingletonService {} modify",
            self.kube_name(),
        ))
    }
    fn delete(&self) -> InstigatorResult {
        let pp = kube::api::DeleteParams::default();
        kube::api::Api::v1Pod(self.meta.client.clone())
            .within(self.meta.namespace.as_str())
            .delete(self.kube_name().as_str(), &pp)?;
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
        )
    }
    fn status(&self) -> StatusResult {
        let pod = match kube::api::Api::v1Pod(self.meta.client.clone())
            .within(self.meta.namespace.as_str())
            .get_status(self.kube_name().as_str())
        {
            Ok(pod) => pod,
            Err(e) => return Ok(e.to_string()),
        };

        let status: api::PodStatus = pod.status.unwrap();
        Ok(status.phase.unwrap_or_else(|| "unknown".to_string()))
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
                annotations: None,
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
        };

        assert_eq!("squidgy", sing.kube_name().as_str());
        assert_eq!(
            "SingletonService",
            sing.labels().get("workload-type").unwrap()
        );
    }

    #[test]
    fn test_singleton_service_to_pod() {
        let cli = APIClient::new(mock_kube_config());
        let mut annotations = BTreeMap::new();
        annotations.insert("key".to_string(), "value".to_string());
        annotations.insert("key2".to_string(), "value2".to_string());

        let sing = SingletonService {
            meta: WorkloadMetadata {
                annotations: Some(annotations),
                name: "de".into(),
                component_name: "hydrate".into(),
                instance_name: "inst".into(),
                namespace: "tests".into(),
                definition: Component {
                    ..Default::default()
                },
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
        };
        let pod = sing.to_pod();
        let pod_annotations = pod
            .metadata
            .clone()
            .expect("metadata")
            .annotations
            .expect("annotations")
            .clone();
        assert_eq!("inst", pod.metadata.expect("metadata").name.expect("name"));
        assert_eq!(2, pod_annotations.len());
        assert_eq!("value", pod_annotations.get("key").expect("a value"));
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
                annotations: None,
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
        };

        assert_eq!("dehydrate", rs.kube_name().as_str());
        assert_eq!("Service", rs.labels().get("workload-type").unwrap());
    }

    #[test]
    fn test_replicated_service_to_deployment() {
        let cli = APIClient::new(mock_kube_config());
        let mut annotations = BTreeMap::new();
        annotations.insert("key".to_string(), "value".to_string());
        annotations.insert("key2".to_string(), "value2".to_string());
        let rs = ReplicatedService {
            meta: WorkloadMetadata {
                name: "name".into(),
                component_name: "component_name".into(),
                instance_name: "instance_name".into(),
                namespace: "namespace".into(),
                definition: Component {
                    ..Default::default()
                },
                annotations: Some(annotations),
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
        };
        let dep = rs.meta.to_deployment("replicated-service");
        let pod_annotations = dep
            .spec
            .expect("spec")
            .template
            .metadata
            .expect("metadata")
            .annotations
            .expect("annotations")
            .clone();
        assert_eq!(
            "instance_name",
            dep.metadata.expect("metadata").name.expect("name")
        );
        assert_eq!(2, pod_annotations.len());
        assert_eq!("value", pod_annotations.get("key").expect("a value"));
    }

    /// This mock builds a KubeConfig that will not be able to make any requests.
    fn mock_kube_config() -> Configuration {
        Configuration {
            base_path: ".".into(),
            client: reqwest::Client::new(),
        }
    }

    #[test]
    fn test_to_config_maps() {
        let mut exp: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        let mut c20 = BTreeMap::new();
        c20.insert("default_user.txt".to_string(), "admin".to_string());
        let mut c21 = BTreeMap::new();
        c21.insert("db-data".to_string(), "test one".to_string());
        exp.insert("container20".to_string(), c20.clone());
        exp.insert("container21".to_string(), c21.clone());
        let cms = to_config_maps(exp, None, None);
        assert_eq!(2, cms.len());
        assert_eq!(
            "container20",
            cms.get(0)
                .unwrap()
                .metadata
                .clone()
                .unwrap()
                .name
                .unwrap()
                .as_str()
        );
        assert_eq!(c20, cms.get(0).unwrap().data.clone().unwrap());
        assert_eq!(c21, cms.get(1).unwrap().data.clone().unwrap());
    }
}
