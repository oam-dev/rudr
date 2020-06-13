use crate::schematic::component::Component;
use crate::workload_type::workload_builder;
use crate::workload_type::{InstigatorResult, ParamMap};
use k8s_openapi::api::apps::v1 as apps;
use k8s_openapi::api::core::v1 as api;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::Client;
use std::collections::BTreeMap;

/// StatefulsetBuilder builds new Singleton Server and Singleton worker use StatefulSet of K8s
///
/// This hides many of the details of building a StatefulSet, exposing only
/// parameters common to Rudr workload types.
pub(crate) struct StatefulsetBuilder {
    component: Component,
    labels: workload_builder::Labels,
    annotations: Option<workload_builder::Labels>,
    name: String,
    restart_policy: String,
    owner_ref: Option<Vec<meta::OwnerReference>>,
    param_vals: ParamMap,
}

impl StatefulsetBuilder {
    /// Create a DeploymentBuilder
    pub fn new(instance_name: String, component: Component) -> Self {
        StatefulsetBuilder {
            component,
            name: instance_name,
            labels: workload_builder::Labels::new(),
            annotations: None,
            restart_policy: "Always".to_string(),
            owner_ref: None,
            param_vals: BTreeMap::new(),
        }
    }
    /// Add labels
    pub fn labels(mut self, labels: workload_builder::Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Add annotations.
    ///
    /// In Kubernetes, these will be added to the pod specification.
    pub fn annotations(mut self, annotations: Option<workload_builder::Labels>) -> Self {
        self.annotations = annotations;
        self
    }

    pub fn parameter_map(mut self, param_vals: ParamMap) -> Self {
        self.param_vals = param_vals;
        self
    }
    /// Set the owner refence for the job and the pod
    pub fn owner_ref(mut self, owner: Option<Vec<meta::OwnerReference>>) -> Self {
        self.owner_ref = owner;
        self
    }

    pub fn to_statefulset(&self) -> apps::StatefulSet {
        apps::StatefulSet {
            metadata: workload_builder::form_metadata(
                self.name.clone(),
                self.labels.clone(),
                self.owner_ref.clone(),
            ),
            spec: Some(apps::StatefulSetSpec {
                selector: meta::LabelSelector {
                    match_labels: Some(self.labels.clone()),
                    ..Default::default()
                },
                template: api::PodTemplateSpec {
                    metadata: Some(meta::ObjectMeta {
                        name: Some(self.name.clone()),
                        labels: Some(self.labels.clone()),
                        annotations: self.annotations.clone(),
                        owner_references: self.owner_ref.clone(),
                        ..Default::default()
                    }),
                    spec: Some(self.component.to_pod_spec_with_policy(
                        self.param_vals.clone(),
                        self.restart_policy.clone(),
                    )),
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub async fn status(self, client: Client, namespace: String) -> Result<String, kube::Error> {
        let api = kube::api::Api::namespaced(client, &namespace);
        let sts: apps::StatefulSet = api.get_status(&self.name).await?;
        let status = sts.status.unwrap();
        let replica = status.replicas;
        let available_replicas = status.ready_replicas.unwrap_or(0);
        let mut state = "updating".to_string();
        if available_replicas == replica {
            state = "running".to_string()
        }
        Ok(state)
    }

    pub async fn do_request(self, client: Client, namespace: String, phase: &str) -> InstigatorResult {
        let statefulset = self.to_statefulset();
        let api: kube::api::Api<apps::StatefulSet> = kube::api::Api::namespaced(client, &namespace);
        match phase {
            "modify" => {
                let pp = kube::api::PatchParams::default();
                api.patch_status(&self.name, &pp, serde_json::to_vec(&statefulset)?).await?;
                Ok(())
            }
            "delete" => {
                let pp = kube::api::DeleteParams::default();
                api.delete(&self.name, &pp).await?;
                Ok(())
            }
            _ => {
                let pp = kube::api::PostParams::default();
                api.replace_status(&self.name, &pp, serde_json::to_vec(&statefulset)?).await?;
                Ok(())
            }
        }
    }
}
