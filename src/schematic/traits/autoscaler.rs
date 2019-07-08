use crate::schematic::traits::{TraitImplementation, util::*};
use crate::workload_type::{ParamMap, REPLICATED_SERVICE_NAME};
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
    pub owner_ref: OwnerRefs,
}

impl Autoscaler {
    pub fn from_params(name: String, instance_name: String, component_name: String, params: ParamMap, owner_ref: OwnerRefs) -> Self {
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
            owner_ref: owner_ref,
        }
    }
    pub fn to_horizontal_pod_autoscaler(&self) -> hpa::HorizontalPodAutoscaler {
        // TODO: fix this to make it configurable
        let metrics = Some(vec![hpa::MetricSpec {
            type_: "Resource".into(),
            resource: Some(hpa::ResourceMetricSource {
                name: "cpu".to_string(),
                target_average_utilization: self.cpu.or(Some(80)),
                target_average_value: None,
            }),
            pods: None,
            object: None,
            external: None,
        }]);
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
                max_replicas: self.maximum.unwrap_or(10),
                metrics: metrics,
                scale_target_ref: hpa::CrossVersionObjectReference {
                    api_version: Some("apps/v1".to_string()),
                    kind: "Deployment".to_string(),
                    name: format!("{}-{}", self.name.as_str(), self.component_name.as_str()),
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
        let res: Result<serde_json::Value, failure::Error> = client.request(req);
        if res.is_err() {
            let err = res.unwrap_err();
            error!(
                "Ingress error: {}",
                serde_json::to_string_pretty(&scaler).expect("debug")
            );
            return Err(err);
        }
        Ok(())
    }
    fn supports_workload_type(name: &str) -> bool {
        // Only support replicated service right now.
        name == REPLICATED_SERVICE_NAME
    }
}