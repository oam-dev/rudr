use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;

use crate::workload_type::workload_builder::{DeploymentBuilder, ServiceBuilder};
use crate::workload_type::{
    InstigatorResult, KubeName, StatusResult, WorkloadMetadata, WorkloadType,
};

use crate::workload_type::statefulset_builder::StatefulsetBuilder;
use std::collections::BTreeMap;

/// A Replicated Server can take one component and scale it up or down.
pub struct ReplicatedServer {
    pub meta: WorkloadMetadata,
}

impl ReplicatedServer {
    fn labels(&self) -> BTreeMap<String, String> {
        self.meta.labels("Service")
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

impl KubeName for ReplicatedServer {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}

impl WorkloadType for ReplicatedServer {
    fn add(&self) -> InstigatorResult {
        //pre create config_map
        self.meta.create_config_maps("Service")?;

        DeploymentBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")?;

        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
            .select_labels(self.meta.select_labels())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")
    }
    fn modify(&self) -> InstigatorResult {
        //TODO update config_map
        DeploymentBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(
                self.meta.client.clone(),
                self.meta.namespace.clone(),
                "modify",
            )?;

        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
            .select_labels(self.meta.select_labels())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(
                self.meta.client.clone(),
                self.meta.namespace.clone(),
                "modify",
            )
    }
    fn delete(&self) -> InstigatorResult {
        DeploymentBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
        )?;

        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
        )
    }
    fn status(&self) -> StatusResult {
        let mut resources = BTreeMap::new();

        let key = "deployment/".to_string() + self.kube_name().as_str();
        let state = self.meta.deployment_status()?;
        resources.insert(key.clone(), state);

        let svc_state = ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .get_status(self.meta.client.clone(), self.meta.namespace.clone());
        let svc_key = "service/".to_string() + self.kube_name().as_str();
        resources.insert(svc_key.clone(), svc_state);

        Ok(resources)
    }
}

/// Singleton represents the Singleton Workload Type, as defined in the OAM specification.
///
/// It is currently implemented as a Kubernetes Statefulset with a Service in front of it.
pub struct SingletonServer {
    pub meta: WorkloadMetadata,
}
impl SingletonServer {
    fn labels(&self) -> BTreeMap<String, String> {
        self.meta.labels("SingletonServer")
    }
}

impl KubeName for SingletonServer {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}
impl WorkloadType for SingletonServer {
    fn add(&self) -> InstigatorResult {
        //pre create config_map
        self.meta.create_config_maps("singleton-service")?;

        // Create deployment
        StatefulsetBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")?;

        // Create service
        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
            .select_labels(self.meta.select_labels())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")
    }

    //TODO: because pod upgrade have many restrictions and very complicated, so we don't support now.
    //User should delete and create a new SingletonServer to solve this.
    fn modify(&self) -> InstigatorResult {
        Err(format_err!(
            "we don't support SingletonServer {} modify",
            self.kube_name(),
        ))
    }
    fn delete(&self) -> InstigatorResult {
        StatefulsetBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
        )?;

        ServiceBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
        )
    }
    fn status(&self) -> StatusResult {
        let mut resources = BTreeMap::new();

        let key = "statefulset/".to_string() + self.kube_name().as_str();
        let state = StatefulsetBuilder::new(self.kube_name(), self.meta.definition.clone())
            .status(self.meta.client.clone(), self.meta.namespace.clone())?;
        resources.insert(key.clone(), state);

        let svc_state = ServiceBuilder::new(self.kube_name(), self.meta.definition.clone())
            .get_status(self.meta.client.clone(), self.meta.namespace.clone());
        let svc_key = "service/".to_string() + self.kube_name().as_str();
        resources.insert(svc_key.clone(), svc_state);

        Ok(resources)
    }
}

#[cfg(test)]
mod test {
    use kube::{client::APIClient, config::Configuration};

    use crate::schematic::component::Component;
    use crate::workload_type::{server::*, KubeName, WorkloadMetadata};

    use std::collections::BTreeMap;

    #[test]
    fn test_singleton_service_kube_name() {
        let cli = APIClient::new(mock_kube_config());

        let sing = SingletonServer {
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
            "SingletonServer",
            sing.labels().get("oam.dev/workload-type").unwrap()
        );
    }

    #[test]
    fn test_replicated_service_kube_name() {
        let cli = APIClient::new(mock_kube_config());

        let rs = ReplicatedServer {
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
        assert_eq!("Service", rs.labels().get("oam.dev/workload-type").unwrap());
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
