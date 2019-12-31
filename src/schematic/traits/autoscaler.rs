use crate::schematic::traits::{util::*, TraitImplementation};
use crate::workload_type::{ParamMap, SERVER_NAME, TASK_NAME, WORKER_NAME};
use k8s_openapi::api::autoscaling::v2beta1 as hpa;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;
use std::collections::BTreeMap;
use serde_json::map::Map;

#[derive(Clone, Debug)]
/// Autoscaler provides autoscaling via a Kubernetes HorizontalPodAutoscaler.
pub struct Autoscaler {
    pub name: String,
    pub instance_name: String,
    pub component_name: String,
    pub minimum: Option<i32>,
    pub maximum: Option<i32>,
    pub cpu: Option<i32>,
    pub memory: Option<i32>,
    pub owner_ref: OwnerRefs,
}

impl Autoscaler {
    pub fn from_params(
        name: String,
        instance_name: String,
        component_name: String,
        params: ParamMap,
        owner_ref: OwnerRefs,
    ) -> Self {
        Autoscaler {
            name,
            component_name,
            instance_name,
            owner_ref,
            minimum: params
                .get("minimum")
                .and_then(|p| p.as_i64().map(|i64| i64 as i32)),
            maximum: params
                .get("maximum")
                .and_then(|p| p.as_i64().map(|i64| i64 as i32)),
            cpu: params
                .get("cpu")
                .and_then(|p| p.as_i64().map(|i64| i64 as i32)),
            memory: params
                .get("memory")
                .and_then(|p| p.as_i64().map(|i64| i64 as i32)),
        }
    }
    pub fn from_properties(
        name: String,
        instance_name: String,
        component_name: String,
        properties_map: Option<&Map<String, serde_json::value::Value>>,
        owner_ref: OwnerRefs,
    ) -> Self {
        Autoscaler {
            name,
            component_name,
            instance_name,
            owner_ref,
            minimum: properties_map
                .and_then(|map| map.get("minimum").and_then(|p| p.as_i64().map(|i64| i64 as i32))),
            maximum: properties_map
                .and_then(|map| map.get("maximum").and_then(|p| p.as_i64().map(|i64| i64 as i32))),
            cpu: properties_map
                .and_then(|map| map.get("cpu").and_then(|p| p.as_i64().map(|i64| i64 as i32))),
            memory: properties_map
                .and_then(|map| map.get("memory").and_then(|p| p.as_i64().map(|i64| i64 as i32))),
        }
    }
    pub fn to_horizontal_pod_autoscaler(&self) -> hpa::HorizontalPodAutoscaler {
        // TODO: fix this to make it configurable
        let mut metrics = Vec::new();

        // Add CPU metrics if set
        self.cpu.and_then(|cpu| {
            metrics.push(hpa::MetricSpec {
                type_: "Resource".into(),
                resource: Some(hpa::ResourceMetricSource {
                    name: "cpu".to_string(),
                    target_average_utilization: Some(cpu),
                    target_average_value: None,
                }),
                pods: None,
                object: None,
                external: None,
            });
            Some(())
        });

        // Add memory metrics if set
        self.memory.and_then(|mem| {
            metrics.push(hpa::MetricSpec {
                type_: "Resource".into(),
                resource: Some(hpa::ResourceMetricSource {
                    name: "memory".to_string(),
                    target_average_utilization: Some(mem),
                    target_average_value: None,
                }),
                pods: None,
                object: None,
                external: None,
            });
            Some(())
        });

        hpa::HorizontalPodAutoscaler {
            metadata: Some(meta::ObjectMeta {
                //name: Some(format!("{}-trait-ingress", self.name.clone())),
                name: Some(self.kube_name()),
                labels: Some(trait_labels(self.name.clone(), self.instance_name.clone())),
                owner_references: self.owner_ref.clone(),
                ..Default::default()
            }),
            spec: Some(hpa::HorizontalPodAutoscalerSpec {
                min_replicas: self.minimum,
                max_replicas: self.maximum.unwrap_or(10 + self.minimum.unwrap_or(0)),
                metrics: Some(metrics),
                scale_target_ref: hpa::CrossVersionObjectReference {
                    api_version: Some("apps/v1".to_string()),
                    kind: "Deployment".to_string(),
                    name: self.instance_name.to_string(),
                },
            }),
            ..Default::default()
        }
    }

    fn kube_name(&self) -> String {
        format!("{}-trait-autoscaler", self.instance_name.as_str())
    }
}

impl TraitImplementation for Autoscaler {
    fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        let scaler = self.to_horizontal_pod_autoscaler();
        let (req, _) = hpa::HorizontalPodAutoscaler::create_namespaced_horizontal_pod_autoscaler(
            ns,
            &scaler,
            Default::default(),
        )?;

        // Deserialize into a Value b/c the response from Kubernetes is not
        // deserializing into an hpa::HorizontalPodAutoscaler correctly.
        let res = client.request::<serde_json::Value>(req)?;
        println!(
            "Autoscaler: {}",
            serde_json::to_string_pretty(&res).unwrap_or_else(|e| e.to_string())
        );
        Ok(())
    }
    fn modify(&self, ns: &str, client: APIClient) -> TraitResult {
        let scaler = self.to_horizontal_pod_autoscaler();

        let values = serde_json::to_value(&scaler)?;
        let (req, _) = hpa::HorizontalPodAutoscaler::patch_namespaced_horizontal_pod_autoscaler(
            self.kube_name().as_str(),
            ns,
            &meta::Patch::StrategicMerge(values),
            Default::default(),
        )?;

        // Deserialize into a Value b/c the response from Kubernetes is not
        // deserializing into an hpa::HorizontalPodAutoscaler correctly.
        let res = client.request::<serde_json::Value>(req)?;
        println!(
            "Autoscaler modified: {}",
            serde_json::to_string_pretty(&res).unwrap_or_else(|e| e.to_string())
        );
        Ok(())
    }
    fn delete(&self, ns: &str, client: APIClient) -> TraitResult {
        let (req, _) = hpa::HorizontalPodAutoscaler::delete_namespaced_horizontal_pod_autoscaler(
            self.kube_name().as_str(),
            ns,
            Default::default(),
        )?;
        client.request::<serde_json::Value>(req)?;
        Ok(())
    }
    fn supports_workload_type(name: &str) -> bool {
        // Only support replicated service and task right now.
        name == SERVER_NAME || name == TASK_NAME || name == WORKER_NAME
    }
    fn status(&self, ns: &str, client: APIClient) -> Option<BTreeMap<String, String>> {
        let mut resource = BTreeMap::new();
        let key = "horizontalpodautoscaler/".to_string() + self.kube_name().as_str();
        let (req, _) =
            match hpa::HorizontalPodAutoscaler::read_namespaced_horizontal_pod_autoscaler_status(
                self.kube_name().as_str(),
                ns,
                Default::default(),
            ) {
                Ok(req) => req,
                Err(e) => {
                    resource.insert(key, e.to_string());
                    return Some(resource);
                }
            };
        let resp: hpa::HorizontalPodAutoscaler =
            match client.request::<hpa::HorizontalPodAutoscaler>(req) {
                Ok(hpa) => hpa,
                Err(e) => {
                    resource.insert(key, e.to_string());
                    return Some(resource);
                }
            };
        if let Some(status) = resp.status {
            resource.insert(key.clone(), status.current_replicas.to_string());
            return Some(resource);
        }
        None
    }
}
