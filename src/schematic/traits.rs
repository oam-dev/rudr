use crate::schematic::parameter::ParameterValue;
use kube::client::APIClient;
use k8s_openapi::api::extensions::v1beta1 as ext;
use k8s_openapi::apimachinery::pkg::{
    apis::meta::v1 as meta,
    util::intstr::IntOrString,
};


/// Trait describes Hydra traits.
/// 
/// Hydra traits are ops-oriented "add-ons" that can be attached to Components of the appropriate workloadType.
/// For example, an autoscaler trait can attach to a workloadType (such as ReplicableService) that can be
/// scaled up and down.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trait {

}

/// A TraitBinding attaches a trait to a component.
/// 
/// Trait bindings appear in configuration stanzas for traits.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TraitBinding {
    pub name: String,
    pub parameter_values: Option<Vec<ParameterValue>>,
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
}

/// An Ingress trait creates an ingress point to the workload type to which it is attached.
/// 
/// In Kubernetes, this will create an Ingress and attach it to the Service of a particular
/// component instance.
pub struct Ingress {
    pub name: String,
    pub svc_port: i32,
    pub hostname: Option<String>,
    pub path: Option<String>,
}
impl Ingress {
    pub fn new(port: i32, name: String, hostname: Option<String>, path: Option<String>) -> Self {
        Ingress{
            name: name,
            hostname: hostname,
            path: path,
            svc_port: port,
        }
    }

    pub fn to_ext_ingress(&self) -> ext::Ingress {
        ext::Ingress {
            metadata: Some(meta::ObjectMeta{
                //name: Some(format!("{}-trait-ingress", self.name.clone())),
                name: Some(self.name.clone()),
                ..Default::default()
            }),
            spec: Some(ext::IngressSpec{
                rules: Some(vec![
                    ext::IngressRule{
                        host: self.hostname.clone().or(Some("example.com".to_string())),
                        http: Some(ext::HTTPIngressRuleValue{
                            paths: vec![ext::HTTPIngressPath{
                                backend: ext::IngressBackend{
                                    service_name: self.name.clone(),
                                    service_port: IntOrString::Int(self.svc_port),
                                },
                                path: self.path.clone().or(Some("/".to_string())),
                            }]
                        }),
                    }
                ]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
impl TraitImplementation for Ingress {
    fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        let ingress = self.to_ext_ingress();
        let (req, _) = ext::Ingress::create_namespaced_ingress(ns, &ingress, Default::default())?;
        let res: Result<serde_json::Value, failure::Error> = client.request(req);
        if res.is_err() {
            let err = res.unwrap_err();
            println!("Ingress error: {}", serde_json::to_string_pretty(&ingress).expect("debug"));
            return Err(err)
        }
        Ok(())
    }
    fn modify(&self) -> TraitResult {
        Err(format_err!("Trait updates not implemented for Ingress"))
    }
    fn delete(&self, ns: &str, client: APIClient) -> TraitResult {
        let (req, _) = ext::Ingress::delete_namespaced_ingress(self.name.as_str(), ns, Default::default())?;
        let res: Result<serde_json::Value, failure::Error> = client.request(req);
        res.and_then(|_| Ok(()))
    }
}