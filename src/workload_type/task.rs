use k8s_openapi::api::batch::v1 as batchapi;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

use crate::schematic::component::Component;
use crate::workload_type::{InstigatorResult, KubeName, ParamMap, WorkloadType, workload_builder::JobBuilder};

use std::collections::BTreeMap;

/// Task represents a non-daemon process that can be parallelized.
///
/// It is currently implemented as a Kubernetes Job.
pub struct ReplicatedTask {
    pub name: String,
    pub component_name: String,
    pub instance_name: String,
    pub namespace: String,
    pub definition: Component,
    pub client: APIClient,
    pub params: ParamMap,
    pub owner_ref: Option<Vec<meta::OwnerReference>>,
    pub replica_count: Option<i32>,
}
impl ReplicatedTask {
    /// Create a Job
    pub fn to_job(&self) -> batchapi::Job {
        let mut labels = BTreeMap::new();
        let podname = self.kube_name();
        labels.insert("app".to_string(), self.name.clone());
        labels.insert("workload-type".to_string(), "task".to_string());
        JobBuilder::new(self.kube_name(), self.definition.clone())
            .labels(labels)
            .parallelism(self.replica_count.unwrap_or(1))
            .owner_ref(self.owner_ref.clone())
            .restart_policy("Never".to_string())
            .to_job()
    }
}

impl KubeName for ReplicatedTask {
    fn kube_name(&self) -> String {
        self.instance_name.to_string()
    }
}
impl WorkloadType for ReplicatedTask {
    fn add(&self) -> InstigatorResult {
        let job = self.to_job();
        let pp = kube::api::PostParams::default();

        // Right now, the Batch API is not transparent through Kube.
        // TODO: Commit upstream
        let batch = kube::api::RawApi {
            group: "batch".into(),
            resource: "jobs".into(),
            prefix: "apis".into(),
            namespace: Some(self.namespace.to_string()),
            version: "v1".into(),
        };

        let req = batch.create(&pp, serde_json::to_vec(&job)?)?;
        self.client.request::<batchapi::Job>(req)?;
        Ok(())
    }
}

/// SingletonTask represents a non-daemon process.
///
/// It is currently implemented as a Kubernetes Job.
pub struct SingletonTask {
    pub name: String,
    pub component_name: String,
    pub instance_name: String,
    pub namespace: String,
    pub definition: Component,
    pub client: APIClient,
    pub params: ParamMap,
    pub owner_ref: Option<Vec<meta::OwnerReference>>,
}
impl SingletonTask {
    /// Create a Job
    pub fn to_job(&self) -> batchapi::Job {
        let mut labels = BTreeMap::new();
        let podname = self.kube_name();
        labels.insert("app".to_string(), self.name.clone());
        labels.insert("workload-type".to_string(), "singleton-task".to_string());
        JobBuilder::new(self.kube_name(), self.definition.clone())
            .labels(labels)
            .owner_ref(self.owner_ref.clone())
            .restart_policy("Never".to_string())
            .to_job()
    }
}

impl KubeName for SingletonTask {
    fn kube_name(&self) -> String {
        self.instance_name.to_string()
    }
}
impl WorkloadType for SingletonTask {
    fn add(&self) -> InstigatorResult {
        let job = self.to_job();
        let pp = kube::api::PostParams::default();

        // Right now, the Batch API is not transparent through Kube.
        // TODO: Commit upstream
        let batch = kube::api::RawApi {
            group: "batch".into(),
            resource: "jobs".into(),
            prefix: "apis".into(),
            namespace: Some(self.namespace.to_string()),
            version: "v1".into(),
        };

        let req = batch.create(&pp, serde_json::to_vec(&job)?)?;
        self.client.request::<batchapi::Job>(req)?;
        Ok(())
    }
}
