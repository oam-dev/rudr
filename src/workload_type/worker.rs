use crate::workload_type::{
    workload_builder::{JobBuilder, WorkloadMetadata},
    InstigatorResult, KubeName, WorkloadType,
};

use std::collections::BTreeMap;

pub struct ReplicatedWorker {
    pub meta: WorkloadMetadata,
    pub replica_count: Option<i32>,
}

impl KubeName for ReplicatedWorker {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}
impl WorkloadType for ReplicatedWorker {
    fn add(&self) -> InstigatorResult {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "worker".to_string());
        JobBuilder::new(self.kube_name(), self.meta.definition.clone())
            .labels(labels)
            .parallelism(self.replica_count.unwrap_or(1))
            .owner_ref(self.meta.owner_ref.clone())
            .restart_policy("OnError".to_string())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone())
    }
}

/// Task represents a non-daemon process.
///
/// It is currently implemented as a Kubernetes Job.
pub struct SingletonWorker {
    pub meta: WorkloadMetadata,
}

impl KubeName for SingletonWorker {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}
impl WorkloadType for SingletonWorker {
    fn add(&self) -> InstigatorResult {
        let mut labels = BTreeMap::new();
        let podname = self.kube_name();
        labels.insert("app".to_string(), self.meta.name.clone());
        labels.insert("workload-type".to_string(), "singleton-worker".to_string());
        JobBuilder::new(podname, self.meta.definition.clone())
            .labels(labels)
            .owner_ref(self.meta.owner_ref.clone())
            .restart_policy("OnError".to_string())
            .do_request(self.meta.client.clone(), self.meta.namespace.clone())
    }
}
