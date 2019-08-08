use crate::schematic::traits::{util::*, TraitImplementation};
use crate::workload_type::{ParamMap, REPLICATED_SERVICE_NAME, REPLICATED_TASK_NAME};
use k8s_openapi::api::autoscaling::v2beta1 as hpa;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

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
            name: name,
            component_name: component_name,
            instance_name: instance_name,
            minimum: params
                .get("minimum".into())
                .and_then(|p| p.as_i64().and_then(|i64| Some(i64 as i32))),
            maximum: params
                .get("maximum")
                .and_then(|p| p.as_i64().and_then(|i| Some(i as i32))),
            cpu: params
                .get("cpu")
                .and_then(|p| p.as_i64().and_then(|i| Some(i as i32))),
            memory: params
                .get("memory")
                .and_then(|p| p.as_i64().and_then(|i| Some(i as i32))),
            owner_ref: owner_ref,
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
                labels: Some(trait_labels()),
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
                    //name: format!("{}-{}", self.name.as_str(), self.component_name.as_str()),
                    name: format!("{}", self.instance_name.as_str()),
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
        println!("Autoscaler: {}", serde_json::to_string_pretty(&res).unwrap_or_else(|e| e.to_string()));
        Ok(())
    }
    fn supports_workload_type(name: &str) -> bool {
        // Only support replicated service and task right now.
        name == REPLICATED_SERVICE_NAME || name == REPLICATED_TASK_NAME
    }
}
