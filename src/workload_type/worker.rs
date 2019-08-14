use k8s_openapi::api::batch::v1 as batchapi;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

use crate::schematic::component::Component;
use crate::workload_type::{
    workload_builder::JobBuilder, InstigatorResult, KubeName, ParamMap, WorkloadType,
};

use std::collections::BTreeMap;

/// Task represents a non-daemon process.
///
/// It is currently implemented as a Kubernetes Job.
pub struct Worker {
    pub name: String,
    pub component_name: String,
    pub instance_name: String,
    pub namespace: String,
    pub definition: Component,
    pub client: APIClient,
    pub params: ParamMap,
    pub replica_count: i32,
    pub owner_ref: Option<Vec<meta::OwnerReference>>,
}
impl Worker {
    /// Create a Job
    pub fn to_job(&self) -> batchapi::Job {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.name.clone());
        labels.insert("workload-type".to_string(), "worker".to_string());
        JobBuilder::new(self.kube_name(), self.definition.clone())
            .labels(labels)
            .parallelism(self.replica_count)
            .owner_ref(self.owner_ref.clone())
            .restart_policy("OnError".to_string())
            .to_job()
    }
}

impl KubeName for Worker {
    fn kube_name(&self) -> String {
        self.instance_name.to_string()
    }
}
impl WorkloadType for Worker {
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

/// Task represents a non-daemon process.
///
/// It is currently implemented as a Kubernetes Job.
pub struct SingletonWorker {
    pub name: String,
    pub component_name: String,
    pub instance_name: String,
    pub namespace: String,
    pub definition: Component,
    pub client: APIClient,
    pub params: ParamMap,
    pub owner_ref: Option<Vec<meta::OwnerReference>>,
}
impl SingletonWorker {
    /// Create a Job
    pub fn to_job(&self) -> batchapi::Job {
        let mut labels = BTreeMap::new();
        let podname = self.kube_name();
        labels.insert("app".to_string(), self.name.clone());
        labels.insert("workload-type".to_string(), "singleton-worker".to_string());
        JobBuilder::new(podname, self.definition.clone())
            .labels(labels)
            .owner_ref(self.owner_ref.clone())
            .restart_policy("OnError".to_string())
            .to_job()
    }
}

impl KubeName for SingletonWorker {
    fn kube_name(&self) -> String {
        self.instance_name.to_string()
    }
}
impl WorkloadType for SingletonWorker {
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