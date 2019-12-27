use crate::schematic::traits::{util::*, TraitImplementation};
use crate::workload_type::ParamMap;
use k8s_openapi::api::extensions::v1beta1 as ext;
use k8s_openapi::apimachinery::pkg::{apis::meta::v1 as meta, util::intstr::IntOrString};
use kube::client::APIClient;
use std::collections::BTreeMap;
use log::{warn};
use serde_json::map::Map;

/// An Ingress trait creates an ingress point to the workload type to which it is attached.
///
/// In Kubernetes, this will create an Ingress and attach it to the Service of a particular
/// component instance.
#[derive(Clone, Debug)]
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
    pub fn from_params(
        name: String,
        instance_name: String,
        component_name: String,
        params: ParamMap,
        owner_ref: OwnerRefs,
    ) -> Self {
        // Right now, we're relying on the higher level validation logic to validate types.
        let instancename = instance_name.clone();
        Ingress {
            name,
            instance_name,
            component_name,
            owner_ref,
            svc_port: params
                .get("service_port")
                .and_then(|p| p.as_i64().map(|p64| p64 as i32))
                .unwrap_or_else(|| params
                    .get("service_port")
                    .and_then(|p| p.as_str().map(|pstr| pstr.parse::<i32>().unwrap_or_else(|_| {
                           warn!("service_port value is provided as string instead of 'int' for the instance:{}. Setting it to default value:80.", instancename); 80
                        })))
                    .unwrap_or_else(|| { warn!("Unable to parse service_port value for instance:{}. Setting it to default value:80", instancename); 80} )
                   ),
            hostname: params
                .get("hostname")
                .map(|p| p.as_str().unwrap_or("").to_string()),
            path: params
                .get("path")
                .map(|p| p.as_str().unwrap_or("").to_string()),
        }
    }
    pub fn from_properties(
        name: String,
        instance_name: String,
        component_name: String,
        properties_map: Option<&Map<String, serde_json::value::Value>>,
        owner_ref: OwnerRefs,
    ) -> Self {
        // Right now, we're relying on the higher level validation logic to validate types.
        let instancename = instance_name.clone();
        Ingress {
            name,
            instance_name,
            component_name,
            owner_ref,
            svc_port: properties_map
                        .and_then(|map| map.get("servicePort").and_then(|p| p.as_i64().map(|p64| p64 as i32))
                        ).unwrap_or_else( || { warn!("Unable to parse service_port value for instance:{}. Setting it to default value:80", instancename); 80}),
            hostname: properties_map
                        .and_then(|map| map.get("hostname").map(|p| p.as_str().unwrap_or("").to_string())),
            path: properties_map
                        .and_then(|map| map.get("path").map(|p| p.as_str().unwrap_or("").to_string()))
        }
    }
    pub fn to_ext_ingress(&self) -> ext::Ingress {
        let labels = trait_labels(self.name.clone(), self.instance_name.clone());
        ext::Ingress {
            metadata: Some(meta::ObjectMeta {
                //name: Some(format!("{}-trait-ingress", self.name.clone())),
                name: Some(self.kube_name()),
                labels: Some(labels),
                owner_references: self.owner_ref.clone(),
                ..Default::default()
            }),
            spec: Some(ext::IngressSpec {
                rules: Some(vec![ext::IngressRule {
                    host: self
                        .hostname
                        .clone()
                        .or_else(|| Some("example.com".to_string())),
                    http: Some(ext::HTTPIngressRuleValue {
                        paths: vec![ext::HTTPIngressPath {
                            backend: ext::IngressBackend {
                                service_name: self.instance_name.clone(),
                                service_port: IntOrString::Int(self.svc_port),
                            },
                            path: self.path.clone().or_else(|| Some("/".to_string())),
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
        client.request::<ext::Ingress>(req)?;
        Ok(())
    }
    fn modify(&self, ns: &str, client: APIClient) -> TraitResult {
        let ingress = self.to_ext_ingress();
        let values = serde_json::to_value(&ingress)?;
        let (req, _) = ext::Ingress::patch_namespaced_ingress(
            self.kube_name().as_str(),
            ns,
            &meta::Patch::StrategicMerge(values),
            Default::default(),
        )?;
        client.request::<ext::Ingress>(req)?;
        Ok(())
    }
    fn delete(&self, ns: &str, client: APIClient) -> TraitResult {
        let (req, _) =
            ext::Ingress::delete_namespaced_ingress(self.name.as_str(), ns, Default::default())?;
        client.request::<ext::Ingress>(req)?;
        Ok(())
    }
    fn status(&self, ns: &str, client: APIClient) -> Option<BTreeMap<String, String>> {
        let mut resource = BTreeMap::new();
        let key = "ingress/".to_string() + self.kube_name().as_str();
        let (req, _) = match ext::Ingress::read_namespaced_ingress_status(
            self.kube_name().as_str(),
            ns,
            Default::default(),
        ) {
            Ok(req) => req,
            Err(e) => {
                resource.insert(key.clone(), e.to_string());
                return Some(resource);
            }
        };
        let ingress = match client.request::<ext::Ingress>(req) {
            Ok(ingress) => ingress,
            Err(e) => {
                resource.insert(key.clone(), e.to_string());
                return Some(resource);
            }
        };

        if let Some(status) = ingress.status {
            if let Some(_lbstatus) = status.load_balancer {
                //we can just put created to status, or combine Hostname and IP to status.
                resource.insert(key.clone(), "created".to_string());
                return Some(resource);
            }
        }
        None
    }
}
