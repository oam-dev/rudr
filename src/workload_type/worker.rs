use crate::workload_type::statefulset_builder::StatefulsetBuilder;
use crate::workload_type::{
    workload_builder::DeploymentBuilder, workload_builder::WorkloadMetadata, InstigatorResult,
    KubeName, StatusResult, ValidationResult, WorkloadType,
};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct ReplicatedWorker {
    pub meta: WorkloadMetadata,
    pub replica_count: Option<i32>,
}

impl ReplicatedWorker {
    fn labels(&self) -> BTreeMap<String, String> {
        self.meta.labels("Worker")
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
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")?;

        Ok(())
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

    fn validate(&self) -> ValidationResult {
        validate_worker(&self.meta)
    }
}

fn validate_worker(meta: &WorkloadMetadata) -> ValidationResult {
    match meta
        .definition
        .containers
        .iter()
        .find(|c| !c.ports.is_empty())
    {
        Some(c) => Err(format_err!(
            "Worker container named {} has a port declared",
            c.name
        )),
        None => Ok(()),
    }
}

/// Task represents a non-daemon process.
///
/// It is currently implemented as a Kubernetes Job.
#[derive(Clone)]
pub struct SingletonWorker {
    pub meta: WorkloadMetadata,
}

impl SingletonWorker {
    fn labels(&self) -> BTreeMap<String, String> {
        self.meta.labels("SingletonWorker")
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

        StatefulsetBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")?;

        Ok(())
    }
    fn modify(&self) -> InstigatorResult {
        //TODO update config_map
        StatefulsetBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .owner_ref(self.meta.owner_ref.clone())
            .do_request(
                self.meta.client.clone(),
                self.meta.namespace.clone(),
                "modify",
            )
    }
    fn delete(&self) -> InstigatorResult {
        StatefulsetBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
        )
    }
    fn status(&self) -> StatusResult {
        let key = "statefulset/".to_string() + self.kube_name().as_str();
        let mut resources = BTreeMap::new();
        let state = StatefulsetBuilder::new(self.kube_name(), self.meta.definition.clone())
            .status(self.meta.client.clone(), self.meta.namespace.clone())?;
        resources.insert(key.clone(), state);
        Ok(resources)
    }
    fn validate(&self) -> ValidationResult {
        validate_worker(&self.meta)
    }
}

#[cfg(test)]
mod test {
    use kube::{client::APIClient, config::Configuration};

    use crate::schematic::component::{Component, Container, Port};
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
                    .get("oam.dev/workload-type")
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
                    definition: Default::default(),
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
                    .get("oam.dev/workload-type")
                    .expect("worker-type label must be set")
            )
        }
    }

    #[test]
    fn test_singleton_worker_validate() {
        let cli = APIClient::new(mock_kube_config());
        let base_worker = SingletonWorker {
            meta: WorkloadMetadata {
                name: "mytask".into(),
                component_name: "workermcworkyface".into(),
                instance_name: "workerinst".into(),
                namespace: "tests".into(),
                definition: Component {
                    containers: vec![Container {
                        name: "good-container".into(),
                        ports: vec![],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                params: BTreeMap::new(),
                client: cli.clone(),
                annotations: None,
                owner_ref: None,
            },
        };
        {
            let mut wrkr = base_worker.clone();
            wrkr.meta.definition.containers.push(Container {
                name: "bad-container".into(),
                ports: vec![
                    Port::basic("http".into(), 80),
                    Port::basic("https".into(), 443),
                ],
                ..Default::default()
            });
            assert!(wrkr.validate().is_err())
        }
        {
            let wrkr = base_worker.clone();
            assert!(wrkr.validate().is_ok())
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
    fn test_replicated_worker_validate() {
        let cli = APIClient::new(mock_kube_config());
        let base_worker = ReplicatedWorker {
            replica_count: Some(132),
            meta: WorkloadMetadata {
                name: "mytask".into(),
                component_name: "workermcworkyface".into(),
                instance_name: "workerinst".into(),
                namespace: "tests".into(),
                definition: Component {
                    containers: vec![Container {
                        name: "good-container".into(),
                        ports: vec![],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                params: BTreeMap::new(),
                client: cli.clone(),
                annotations: None,
                owner_ref: None,
            },
        };
        {
            let mut wrkr = base_worker.clone();
            wrkr.meta.definition.containers.push(Container {
                name: "bad-container".into(),
                ports: vec![
                    Port::basic("http".into(), 80),
                    Port::basic("https".into(), 443),
                ],
                ..Default::default()
            });
            assert!(wrkr.validate().is_err())
        }
        {
            let wrkr = base_worker.clone();
            assert!(wrkr.validate().is_ok())
        }
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
