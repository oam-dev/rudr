use crate::workload_type::{
    workload_builder::{JobBuilder, WorkloadMetadata},
    InstigatorResult, KubeName, WorkloadType,
};

use std::collections::BTreeMap;

/// Task represents a non-daemon process that can be parallelized.
///
/// It is currently implemented as a Kubernetes Job.
pub struct ReplicatedTask {
    pub meta: WorkloadMetadata,
    pub replica_count: Option<i32>,
}
impl KubeName for ReplicatedTask {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}
impl WorkloadType for ReplicatedTask {
    fn add(&self) -> InstigatorResult {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "task".to_string());
        JobBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(labels)
            .parallelism(self.replica_count.unwrap_or(1))
            .owner_ref(self.meta.owner_ref.clone())
            .restart_policy("Never".to_string())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone())
    }
}

/// SingletonTask represents a non-daemon process.
///
/// It is currently implemented as a Kubernetes Job.
pub struct SingletonTask {
    pub meta: WorkloadMetadata,
}
impl KubeName for SingletonTask {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}
impl WorkloadType for SingletonTask {
    fn add(&self) -> InstigatorResult {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "singleton-task".to_string());
        JobBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(labels)
            .owner_ref(self.meta.owner_ref.clone())
            .restart_policy("Never".to_string())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone())
    }
}

#[cfg(test)]
mod test {
    use kube::{client::APIClient, config::Configuration};

    use crate::schematic::component::Component;
    use crate::workload_type::{task::*, workload_builder::WorkloadMetadata, KubeName};

    use std::collections::BTreeMap;

    #[test]
    fn test_singleton_task_kube_name() {
        let cli = APIClient::new(mock_kube_config());

        let task = SingletonTask {
            meta: WorkloadMetadata {
                name: "mytask".into(),
                component_name: "taskrunner".into(),
                instance_name: "taskinstance".into(),
                namespace: "tests".into(),
                definition: Component {
                    ..Default::default()
                },
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
        };

        assert_eq!("taskinstance", task.kube_name().as_str());
    }

    #[test]
    fn test_replicated_task_kube_name() {
        let cli = APIClient::new(mock_kube_config());

        let task = ReplicatedTask {
            meta: WorkloadMetadata {
                name: "mytask".into(),
                component_name: "taskrunner".into(),
                instance_name: "taskinstance".into(),
                namespace: "tests".into(),
                definition: Component {
                    ..Default::default()
                },
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
            replica_count: Some(1),
        };

        assert_eq!("taskinstance", task.kube_name().as_str());
    }

    /// This mock builds a KubeConfig that will not be able to make any requests.
    fn mock_kube_config() -> Configuration {
        Configuration {
            base_path: ".".into(),
            client: reqwest::Client::new(),
        }
    }

}
