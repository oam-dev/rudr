use crate::workload_type::{
    workload_builder::DeploymentBuilder, workload_builder::WorkloadMetadata, InstigatorResult,
    KubeName, StatusResult, WorkloadType,
};
use std::collections::BTreeMap;

pub struct ReplicatedWorker {
    pub meta: WorkloadMetadata,
    pub replica_count: Option<i32>,
}

impl ReplicatedWorker {
    fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "Worker".to_string());
        labels
    }
}

impl KubeName for ReplicatedWorker {
    fn kube_name(&self) -> String {
        self.meta.kube_name()
    }
}

impl WorkloadType for ReplicatedWorker {
    fn add(&self) -> InstigatorResult {
        //pre create config_map
        self.meta.create_config_maps("Worker")?;

        DeploymentBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")?;

        Ok(())
    }
    fn modify(&self) -> InstigatorResult {
        //TODO update config_map
        DeploymentBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
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
        )
    }
    fn status(&self) -> StatusResult {
        let key = "deployment/".to_string() + self.kube_name().as_str();
        let mut resources = BTreeMap::new();
        let state = self.meta.deployment_status()?;
        resources.insert(key.clone(), state);
        Ok(resources)
    }
}

/// Task represents a non-daemon process.
///
/// It is currently implemented as a Kubernetes Job.
pub struct SingletonWorker {
    pub meta: WorkloadMetadata,
}

impl SingletonWorker {
    fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "SingletonWorker".to_string());
        labels
    }
}

impl KubeName for SingletonWorker {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}

impl WorkloadType for SingletonWorker {
    fn add(&self) -> InstigatorResult {
        //pre create config_map
        self.meta.create_config_maps("SingletonWorker")?;

        DeploymentBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")?;

        Ok(())
    }
    fn modify(&self) -> InstigatorResult {
        //TODO update config_map
        DeploymentBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(self.labels())
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
        )
    }
    fn status(&self) -> StatusResult {
        let key = "deployment/".to_string() + self.kube_name().as_str();
        let mut resources = BTreeMap::new();
        let state = self.meta.deployment_status()?;
        resources.insert(key.clone(), state);
        Ok(resources)
    }
}

#[cfg(test)]
mod test {
    use kube::{client::APIClient, config::Configuration};

    use crate::schematic::component::Component;
    use crate::workload_type::{worker::*, workload_builder::WorkloadMetadata, KubeName};

    use std::collections::BTreeMap;

    #[test]
    fn test_worker_labels() {
        {
            let wrkr = ReplicatedWorker {
                meta: WorkloadMetadata {
                    name: "mytask".into(),
                    component_name: "workerbee".into(),
                    instance_name: "workerinst".into(),
                    namespace: "tests".into(),
                    definition: Component {
                        ..Default::default()
                    },
                    annotations: None,
                    params: BTreeMap::new(),
                    client: APIClient::new(mock_kube_config()),
                    owner_ref: None,
                },
                replica_count: Some(1),
            };

            let labels = wrkr.labels();
            assert_eq!(
                "Worker",
                labels
                    .get("workload-type")
                    .expect("worker-type label must be set")
            )
        }
        {
            let wrkr = SingletonWorker {
                meta: WorkloadMetadata {
                    name: "mytask".into(),
                    component_name: "workerbee".into(),
                    instance_name: "workerinst".into(),
                    namespace: "tests".into(),
                    definition: Component {
                        ..Default::default()
                    },
                    annotations: None,
                    params: BTreeMap::new(),
                    client: APIClient::new(mock_kube_config()),
                    owner_ref: None,
                },
            };

            let labels = wrkr.labels();
            assert_eq!(
                "SingletonWorker",
                labels
                    .get("workload-type")
                    .expect("worker-type label must be set")
            )
        }
    }

    #[test]
    fn test_singleton_worker_kube_name() {
        let cli = APIClient::new(mock_kube_config());

        let wrkr = SingletonWorker {
            meta: WorkloadMetadata {
                name: "mytask".into(),
                component_name: "workermcworkyface".into(),
                instance_name: "workerinst".into(),
                namespace: "tests".into(),
                definition: Component {
                    ..Default::default()
                },
                params: BTreeMap::new(),
                client: cli,
                annotations: None,
                owner_ref: None,
            },
        };

        assert_eq!("workerinst", wrkr.kube_name().as_str());
    }

    #[test]
    fn test_replicated_worker_kube_name() {
        let cli = APIClient::new(mock_kube_config());
        let mut annotations = BTreeMap::new();
        annotations.insert("annotation1".to_string(), "value".to_string());

        let wrkr = ReplicatedWorker {
            meta: WorkloadMetadata {
                name: "mytask".into(),
                component_name: "workerbee".into(),
                instance_name: "workerinst".into(),
                namespace: "tests".into(),
                definition: Component {
                    ..Default::default()
                },
                annotations: Some(annotations),
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
            replica_count: Some(1),
        };

        assert_eq!("workerinst", wrkr.kube_name().as_str());
    }

    /// This mock builds a KubeConfig that will not be able to make any requests.
    fn mock_kube_config() -> Configuration {
        Configuration {
            base_path: ".".into(),
            client: reqwest::Client::new(),
        }
    }

}
