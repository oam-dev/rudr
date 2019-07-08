use crate::schematic::traits::{TraitImplementation, util::*};
use crate::workload_type::ParamMap;
use k8s_openapi::api::extensions::v1beta1 as ext;
use k8s_openapi::apimachinery::pkg::{apis::meta::v1 as meta, util::intstr::IntOrString};
use kube::client::APIClient;

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
    pub owner_ref: OwnerRefs,
}
impl Ingress {
    pub fn from_params(name: String, instance_name: String, component_name: String, params: ParamMap, owner_ref: OwnerRefs) -> Self {
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
            owner_ref: owner_ref,
        }
    }
    pub fn to_ext_ingress(&self) -> ext::Ingress {
        ext::Ingress {
            metadata: Some(meta::ObjectMeta {
                //name: Some(format!("{}-trait-ingress", self.name.clone())),
                name: Some(self.kube_name()),
                labels: Some(trait_labels()),
                owner_references: self.owner_ref.clone(),
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
}
