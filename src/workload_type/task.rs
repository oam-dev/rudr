use async_trait::async_trait;
use crate::workload_type::{
    workload_builder::{JobBuilder, WorkloadMetadata},
    InstigatorResult, KubeName, StatusResult, WorkloadType,
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

impl ReplicatedTask {
    fn labels(&self) -> BTreeMap<String, String> {
        self.meta.labels("Task")
    }
}

#[async_trait]
impl WorkloadType for ReplicatedTask {
    async fn add(&self) -> InstigatorResult {
        JobBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .parallelism(self.replica_count.unwrap_or(1))
            .owner_ref(self.meta.owner_ref.clone())
            .restart_policy("Never".to_string())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")
            .await
    }
    async fn modify(&self) -> InstigatorResult {
        JobBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .parallelism(self.replica_count.unwrap_or(1))
            .owner_ref(self.meta.owner_ref.clone())
            .restart_policy("Never".to_string())
            .do_request(
                self.meta.client.clone(),
                self.meta.namespace.clone(),
                "modify",
            ).await
    }
    async fn delete(&self) -> InstigatorResult {
        JobBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
        ).await
    }
    async fn status(&self) -> StatusResult {
        let mut resources = BTreeMap::new();
        let key = "job/".to_string() + self.kube_name().as_str();
        let state = JobBuilder::new(self.kube_name(), self.meta.definition.clone())
            .get_status(self.meta.client.clone(), self.meta.namespace.clone()).await;
        resources.insert(key.clone(), state);

        Ok(resources)
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
impl SingletonTask {
    fn labels(&self) -> BTreeMap<String, String> {
        self.meta.labels("SingletonTask")
    }
}

#[async_trait]
impl WorkloadType for SingletonTask {
    async fn add(&self) -> InstigatorResult {
        JobBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .owner_ref(self.meta.owner_ref.clone())
            .restart_policy("Never".to_string())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone(), "add")
            .await
    }
    async fn modify(&self) -> InstigatorResult {
        JobBuilder::new(self.kube_name(), self.meta.definition.clone())
            .parameter_map(self.meta.params.clone())
            .labels(self.labels())
            .annotations(self.meta.annotations.clone())
            .owner_ref(self.meta.owner_ref.clone())
            .restart_policy("Never".to_string())
            .do_request(
                self.meta.client.clone(),
                self.meta.namespace.clone(),
                "modify",
            ).await
    }
    async fn delete(&self) -> InstigatorResult {
        JobBuilder::new(self.kube_name(), self.meta.definition.clone()).do_request(
            self.meta.client.clone(),
            self.meta.namespace.clone(),
            "delete",
        ).await
    }
    async fn status(&self) -> StatusResult {
        let mut resources = BTreeMap::new();
        let key = "job/".to_string() + self.kube_name().as_str();
        let state = JobBuilder::new(self.kube_name(), self.meta.definition.clone())
            .get_status(self.meta.client.clone(), self.meta.namespace.clone()).await;
        resources.insert(key.clone(), state);

        Ok(resources)
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
                annotations: None,
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
                annotations: None,
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
            replica_count: Some(1),
        };

        assert_eq!("taskinstance", task.kube_name().as_str());
        assert_eq!("Task", task.labels().get("oam.dev/workload-type").unwrap());
    }

    /// This mock builds a KubeConfig that will not be able to make any requests.
    fn mock_kube_config() -> Configuration {
        Configuration::new(
            ".".into(),
            reqwest::Client::new(),
        )
    }

}
