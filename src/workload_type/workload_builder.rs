use k8s_openapi::api::batch::v1 as batchapi;
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use std::collections::BTreeMap;

use crate::schematic::component::Component;

type Labels = BTreeMap<String, String>;

/// JobBuilder builds new jobs specific to Scylla
/// 
/// This hides many of the details of building a Job, exposing only
/// parameters common to Scylla workload types.
pub(crate) struct JobBuilder {
    component: Component,
    labels: Labels,
    name: String,
    restart_policy: String,
    owner_ref: Option<Vec<meta::OwnerReference>>,
    parallelism: Option<i32>,
}

impl JobBuilder {
    /// Create a JobBuilder
    pub fn new(instance_name: String, component: Component) -> Self {
        JobBuilder {
            name: instance_name,
            component: component,
            labels: BTreeMap::new(),
            restart_policy: "Never".to_string(),
            owner_ref: None,
            parallelism: None,
        }
    }
    /// Add labels
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }
    /// Set the restart policy
    pub fn restart_policy(mut self, policy: String) -> Self {
        self.restart_policy = policy;
        self
    }
    /// Set the owner refence for the job and the pod
    pub fn owner_ref(mut self, owner: Option<Vec<meta::OwnerReference>>) -> Self {
        self.owner_ref = owner;
        self
    }
    /// Set the parallelism
    pub fn parallelism(mut self, count: i32) -> Self {
        self.parallelism = Some(count);
        self
    }
    pub fn to_job(self) -> batchapi::Job {
        batchapi::Job {
            // TODO: Could make this generic.
            metadata: Some(meta::ObjectMeta {
                name: Some(self.name.clone()),
                labels: Some(self.labels.clone()),
                owner_references: self.owner_ref.clone(),
                ..Default::default()
            }),
            spec: Some(batchapi::JobSpec {
                backoff_limit: Some(4),
                parallelism: self.parallelism,
                template: api::PodTemplateSpec {
                    metadata: Some(meta::ObjectMeta {
                        name: Some(self.name.clone()),
                        labels: Some(self.labels.clone()),
                        owner_references: self.owner_ref.clone(),
                        ..Default::default()
                    }),
                    spec: Some(
                        self.component
                            .to_pod_spec_with_policy(self.restart_policy.clone()),
                    ),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

/*
pub fn to_job(&self) -> batchapi::Job {
        let mut labels = BTreeMap::new();
        let podname = self.kube_name();
        labels.insert("app".to_string(), self.name.clone());
        labels.insert("workload-type".to_string(), "worker".to_string());
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
                    spec: Some(self.definition.to_pod_spec_with_policy("OnFailure".into())),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }
    */
