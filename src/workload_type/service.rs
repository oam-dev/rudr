use k8s_openapi::api::apps::v1 as apps;
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::api::{PatchParams, PostParams};

use crate::workload_type::workload_builder::ServiceBuilder;
use crate::workload_type::{InstigatorResult, KubeName, WorkloadMetadata, WorkloadType};

use std::collections::BTreeMap;

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
    fn to_config_maps(&self) -> Vec<api::ConfigMap> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert(
            "workload-type".to_string(),
            "replicated-service".to_string(),
        );
        let configs = self
            .meta
            .definition
            .evaluate_configs(self.meta.params.clone());

        to_config_maps(configs, self.meta.owner_ref.clone(), Some(labels.clone()))
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
        let config_maps = self.to_config_maps();
        if !config_maps.is_empty() {
            log::debug!("start to create {} config_maps", config_maps.len());
        }
        for config in config_maps.iter() {
            let (req, _) = api::ConfigMap::create_namespaced_config_map(
                self.meta.namespace.as_str(),
                config,
                Default::default(),
            )?;
            self.meta.client.request::<api::ConfigMap>(req)?;
        }

        let deployment = self.to_deployment();
        let deployments = kube::api::Api::v1Deployment(self.meta.client.clone())
            .within(self.meta.namespace.as_str());
        let pp = PostParams::default();
        deployments.create(&pp, serde_json::to_vec(&deployment)?)?;

        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "service".to_string());
        let mut select_labels = BTreeMap::new();
        select_labels.insert("app".to_string(), self.meta.name.clone());
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(labels)
            .select_labels(select_labels)
            .owner_reference(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")
    }
    fn modify(&self) -> InstigatorResult {
        //TODO update config_map
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
    fn delete(&self) -> InstigatorResult {
        let pp = kube::api::DeleteParams::default();
        kube::api::Api::v1Deployment(self.meta.client.clone())
            .within(self.meta.namespace.as_str())
            .delete(self.kube_name().as_str(), &pp)?;
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
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
    fn to_config_maps(&self) -> Vec<api::ConfigMap> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert(
            "workload-type".to_string(),
            "replicated-service".to_string(),
        );
        let configs = self
            .meta
            .definition
            .evaluate_configs(self.meta.params.clone());

        to_config_maps(configs, self.meta.owner_ref.clone(), Some(labels.clone()))
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
        let config_maps = self.to_config_maps();
        for config in config_maps.iter() {
            let (req, _) = api::ConfigMap::create_namespaced_config_map(
                self.meta.namespace.as_str(),
                config,
                Default::default(),
            )?;
            self.meta.client.request::<api::ConfigMap>(req)?;
        }

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
