use k8s_openapi::api::batch::v1 as batchapi;
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

use crate::schematic::component::Component;
use crate::workload_type::{InstigatorResult, KubeName, ParamMap, WorkloadType};

use std::collections::BTreeMap;

/// ReplicatedTask represents a non-daemon process that can be parallelized.
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
        batchapi::Job {
            // TODO: Could make this generic.
            metadata: Some(meta::ObjectMeta {
                name: Some(podname.clone()),
                labels: Some(labels.clone()),
                owner_references: self.owner_ref.clone(),
                ..Default::default()
            }),
            spec: Some(batchapi::JobSpec {
                backoff_limit: Some(4),
                parallelism: self.replica_count,
                template: api::PodTemplateSpec {
                    metadata: Some(meta::ObjectMeta {
                        name: Some(podname),
                        labels: Some(labels),
                        owner_references: self.owner_ref.clone(),
                        ..Default::default()
                    }),
                    spec: Some(self.definition.to_pod_spec_with_policy("Never".into())),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
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
        let (req, _) = batchapi::Job::create_namespaced_job(
            self.namespace.as_str(),
            &job,
            Default::default(),
        )?;

        // We force the decoded value into a serde_json::Value because we don't care if Kubernetes returns a
        // malformed body. We just want the response code validated by APIClient.
        let res: Result<serde_json::Value, failure::Error> = self.client.request(req);
        res.and(Ok(()))
    }
}
