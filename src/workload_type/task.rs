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
            .labels(labels)
            .owner_ref(self.meta.owner_ref.clone())
            .restart_policy("Never".to_string())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone())
    }
}
