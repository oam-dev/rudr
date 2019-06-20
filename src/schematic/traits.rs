use crate::schematic::parameter::ParameterValue;
use crate::workload_type::{ParamMap, REPLICATED_SERVICE_NAME};
use k8s_openapi::api::{autoscaling::v2beta1 as hpa, extensions::v1beta1 as ext};
use k8s_openapi::apimachinery::pkg::{apis::meta::v1 as meta, util::intstr::IntOrString};
use kube::client::APIClient;
use std::collections::BTreeMap;

type Labels = BTreeMap<String, String>;

/// Trait describes Hydra traits.
///
/// Hydra traits are ops-oriented "add-ons" that can be attached to Components of the appropriate workloadType.
/// For example, an autoscaler trait can attach to a workloadType (such as ReplicableService) that can be
/// scaled up and down.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trait {}

/// A TraitBinding attaches a trait to a component.
///
/// Trait bindings appear in configuration stanzas for traits.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TraitBinding {
    pub name: String,
    pub parameter_values: Option<Vec<ParameterValue>>,
}

/// HydraTrait is an enumeration of the known traits.
///
/// This is a temporary solution. In the future, we really want to be able to proxy
/// trait fulfillment down into Kubernetes and let individual trait controllers
/// fulfill the contract.
pub enum HydraTrait {
    Autoscaler(Autoscaler),
    Ingress(Ingress),
}
impl HydraTrait {
    pub fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        match self {
            HydraTrait::Autoscaler(a) => a.add(ns, client),
            HydraTrait::Ingress(i) => i.add(ns, client),
        }
    }
    pub fn delete(&self, ns: &str, client: APIClient) -> TraitResult {
        match self {
            HydraTrait::Autoscaler(a) => a.delete(ns, client),
            HydraTrait::Ingress(i) => i.delete(ns, client),
        }
    }
    pub fn modify(&self) -> TraitResult {
        match self {
            HydraTrait::Autoscaler(a) => a.modify(),
            HydraTrait::Ingress(i) => i.modify(),
        }
    }
}

/// Alias for trait results.
type TraitResult = Result<(), failure::Error>;

/// A TraitImplementation is an implementation of a Hydra Trait.
///
/// For example, Ingress is an implementation of a Hydra Trait.
pub trait TraitImplementation {
    fn add(&self, ns: &str, client: APIClient) -> TraitResult;
    fn modify(&self) -> TraitResult;
    fn delete(&self, ns: &str, client: APIClient) -> TraitResult;
    fn supports_workload_type(name: &str) -> bool;
}

/// Generate the common labels for a trait.
pub fn trait_labels() -> Labels {
    let mut labels: Labels = BTreeMap::new();
    labels.insert("hydra.io/role".into(), "trait".into());
    labels
}

/// An Ingress trait creates an ingress point to the workload type to which it is attached.
///
/// In Kubernetes, this will create an Ingress and attach it to the Service of a particular
/// component instance.
pub struct Ingress {
    pub name: String,
    pub instance_name: String,
    pub component_name: String,
    pub svc_port: i32,
    pub hostname: Option<String>,
    pub path: Option<String>,
}
impl Ingress {
    pub fn from_params(name: String, instance_name: String, component_name: String, params: ParamMap) -> Self {
        // Right now, we're relying on the higher level validation logic to validate types.
        Ingress {
            name: name,
            instance_name: instance_name,
            component_name: component_name,
            svc_port: params
                .get("service_port".into())
                .and_then(|p| p.as_i64().and_then(|p64| Some(p64 as i32)))
                .unwrap_or(80),
            hostname: params
                .get("hostname".into())
                .and_then(|p| Some(p.to_string())),
            path: params.get("path".into()).and_then(|p| Some(p.to_string())),
        }
    }
    pub fn to_ext_ingress(&self) -> ext::Ingress {
        ext::Ingress {
            metadata: Some(meta::ObjectMeta {
                //name: Some(format!("{}-trait-ingress", self.name.clone())),
                name: Some(self.kube_name()),
                labels: Some(trait_labels()),
                ..Default::default()
            }),
            spec: Some(ext::IngressSpec {
                rules: Some(vec![ext::IngressRule {
                    host: self.hostname.clone().or(Some("example.com".to_string())),
                    http: Some(ext::HTTPIngressRuleValue {
                        paths: vec![ext::HTTPIngressPath {
                            backend: ext::IngressBackend {
                                service_name: self.name.clone(),
                                service_port: IntOrString::Int(self.svc_port),
                            },
                            path: self.path.clone().or(Some("/".to_string())),
                        }],
                    }),
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
    fn kube_name(&self) -> String {
        format!("{}-trait-ingress", self.instance_name)
    }
}
impl TraitImplementation for Ingress {
    fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        let ingress = self.to_ext_ingress();
        let (req, _) = ext::Ingress::create_namespaced_ingress(ns, &ingress, Default::default())?;
        let res: Result<serde_json::Value, failure::Error> = client.request(req);
        if res.is_err() {
            let err = res.unwrap_err();
            error!(
                "Ingress error: {}",
                serde_json::to_string_pretty(&ingress).expect("debug")
            );
            return Err(err);
        }
        Ok(())
    }
    fn modify(&self) -> TraitResult {
        Err(format_err!("Trait updates not implemented for Ingress"))
    }
    fn delete(&self, ns: &str, client: APIClient) -> TraitResult {
        let (req, _) = ext::Ingress::delete_namespaced_ingress(
            self.kube_name().as_str(),
            ns,
            Default::default(),
        )?;
        let res: Result<serde_json::Value, failure::Error> = client.request(req);
        res.and_then(|_| Ok(()))
    }
    fn supports_workload_type(_name: &str) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
/// Autoscaler provides autoscaling via a Kubernetes HorizontalPodAutoscaler.
pub struct Autoscaler {
    pub name: String,
    pub instance_name: String,
    pub component_name: String,
    pub minimum: Option<i32>,
    pub maximum: Option<i32>,
    pub cpu: Option<i32>,
}

impl Autoscaler {
    pub fn from_params(name: String, instance_name: String, component_name: String, params: ParamMap) -> Self {
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
    fn modify(&self) -> TraitResult {
        Err(format_err!("Trait updates not implemented for Autoscaler"))
    }
    fn delete(&self, ns: &str, client: APIClient) -> TraitResult {
        let (req, _) = hpa::HorizontalPodAutoscaler::delete_namespaced_horizontal_pod_autoscaler(
            self.kube_name().as_str(),
            ns,
            Default::default(),
        )?;
        let res: Result<serde_json::Value, failure::Error> = client.request(req);
        res.and_then(|_| Ok(()))
    }
    fn supports_workload_type(name: &str) -> bool {
        // Only support replicated service right now.
        name == REPLICATED_SERVICE_NAME
    }
}
