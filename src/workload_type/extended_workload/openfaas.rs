use crate::schematic::parameter::resolve_value;
use crate::workload_type::{
    InstigatorResult, StatusResult, ValidationResult, WorkloadMetadata, WorkloadType,
};
use failure::{format_err, Error};
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::api::{Object, ObjectMeta, PostParams, RawApi, TypeMeta};
use log::info;
use std::collections::BTreeMap;

pub const OPENFAAS: &str = "openfaas.com/v1alpha2.Function";

// FunctionSpec is the spec for a Function resource
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct FunctionSpec {
    pub name: String,
    pub image: String,
    pub replicas: Option<i32>,
    pub handler: Option<String>,
    pub annotations: Option<BTreeMap<String, String>>,
    pub labels: Option<BTreeMap<String, String>>,
    pub environment: Option<BTreeMap<String, String>>,
    pub constraints: Option<Vec<String>>,
    pub secrets: Option<Vec<String>>,
    pub limits: Option<FunctionResources>,
    pub requests: Option<FunctionResources>,
    pub read_only_root_filesystem: Option<bool>,
}

impl Default for FunctionSpec {
    fn default() -> Self {
        FunctionSpec {
            name: "".to_string(),
            image: "".to_string(),
            replicas: None,
            handler: None,
            annotations: None,
            labels: None,
            environment: None,
            constraints: None,
            secrets: None,
            limits: None,
            requests: None,
            read_only_root_filesystem: None,
        }
    }
}

// FunctionResources is used to set CPU and memory limits and requests
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct FunctionResources {
    pub memory: Option<String>,
    pub cpu: Option<String>,
}

impl Default for FunctionResources {
    fn default() -> Self {
        FunctionResources {
            memory: None,
            cpu: None,
        }
    }
}

// FunctionStatus is the status for a Function resource
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct FunctionStatus {
    pub available_replicas: Option<i32>,
}
impl Default for FunctionStatus {
    fn default() -> Self {
        FunctionStatus {
            available_replicas: None,
        }
    }
}

type KubeFaaS = Object<FunctionSpec, FunctionStatus>;

pub struct OpenFaaS {
    pub meta: WorkloadMetadata,
}

impl OpenFaaS {
    pub fn extract_environment(&self) -> Option<BTreeMap<String, String>> {
        let mut envs = BTreeMap::new();
        let array = match self
            .meta
            .get_workload_setting("environment")
            .and_then(|im| im.as_array().cloned())
        {
            None => return None,
            Some(ar) => ar,
        };

        for element in array.iter() {
            let mp = match element.as_object() {
                None => continue,
                Some(mp) => mp.clone(),
            };
            let key = mp.get("name");
            let value = mp.get("value");
            let from_param = mp.get("fromParam");
            let real_value = from_param.and_then(|fp| {
                fp.as_str().and_then(|s| {
                    resolve_value(
                        self.meta.params.clone(),
                        Some(s.to_string()),
                        value.and_then(|v| Some(v.clone())),
                    )
                })
            });
            key.and_then(|k| {
                k.as_str().and_then(|s| {
                    real_value.and_then(|rv| {
                        rv.as_str()
                            .and_then(|v| envs.insert(s.to_string(), v.to_string()))
                    })
                })
            });
        }

        if envs.is_empty() {
            None
        } else {
            Some(envs)
        }
    }
    pub fn get_kube_faas(&self) -> Result<KubeFaaS, Error> {
        let image = match self
            .meta
            .get_workload_setting("image")
            .and_then(|im| im.as_str().and_then(|s| Some(s.to_string())))
        {
            None => {
                return Err(format_err!(
                    "image is required in workload setting and must be string",
                ))
            }
            Some(im) => im,
        };
        let handler = self
            .meta
            .get_workload_setting("handler")
            .and_then(|im| im.as_str().and_then(|s| Some(s.to_string())));

        let mut kube_faas = KubeFaaS {
            types: TypeMeta {
                apiVersion: Some("openfaas.com/v1alpha2".to_string()),
                kind: Some("Function".to_string()),
            },
            metadata: ObjectMeta {
                name: self.meta.instance_name.clone(),
                ..Default::default()
            },
            spec: FunctionSpec {
                name: self.meta.instance_name.clone(),
                image: image.to_string(),
                handler,
                environment: self.extract_environment(),
                ..Default::default()
            },
            status: None,
        };
        if self.meta.owner_ref.is_some() {
            kube_faas.metadata.ownerReferences =
                convert_owner_ref(self.meta.owner_ref.clone().unwrap());
        }
        Ok(kube_faas)
    }
}

fn convert_owner_ref(own: Vec<meta::OwnerReference>) -> Vec<kube::api::OwnerReference> {
    let mut own_ref = vec![];
    if own.is_empty() {
        return own_ref;
    }
    for o in own.iter() {
        own_ref.insert(
            own_ref.len(),
            kube::api::OwnerReference {
                apiVersion: o.api_version.clone(),
                kind: o.kind.clone(),
                name: o.name.clone(),
                uid: o.uid.clone(),
                blockOwnerDeletion: o.block_owner_deletion.unwrap_or_else(|| false),
                controller: o.controller.unwrap_or_else(|| false),
            },
        );
    }
    own_ref
}

impl WorkloadType for OpenFaaS {
    fn add(&self) -> InstigatorResult {
        let faas_resource = RawApi::customResource("functions")
            .version("v1alpha2")
            .group("openfaas.com")
            .within(self.meta.namespace.as_str());
        let kubefaas = self.get_kube_faas()?;
        let faas_req =
            faas_resource.create(&PostParams::default(), serde_json::to_vec(&kubefaas)?)?;
        let openfaas: KubeFaaS = self.meta.client.request(faas_req)?;
        info!("openfass function {} was created", openfaas.metadata.name);
        Ok(())
    }
    fn modify(&self) -> InstigatorResult {
        Ok(())
    }
    fn delete(&self) -> InstigatorResult {
        Ok(())
    }
    fn status(&self) -> StatusResult {
        Ok(BTreeMap::new())
    }
    fn validate(&self) -> ValidationResult {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::schematic::component::{Component, WorkloadSetting};
    use crate::schematic::parameter::ParameterType;
    use crate::workload_type::extended_workload::openfaas::OpenFaaS;
    use crate::workload_type::{ParamMap, WorkloadMetadata};
    use kube::client::APIClient;
    use kube::config::Configuration;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn test_extract_env() {
        let mut params = ParamMap::new();
        params.insert(
            "write_debug".to_string(),
            serde_json::to_value("false").unwrap(),
        );
        let of = OpenFaaS {
            meta: WorkloadMetadata {
                name: "test".to_string(),
                component_name: "test".to_string(),
                instance_name: "test".to_string(),
                namespace: "default".to_string(),
                definition: Component {
                    workload_settings: vec![WorkloadSetting {
                        name: "environment".to_string(),
                        parameter_type: ParameterType::Array,
                        value: Some(
                            serde_json::to_value(json!([
                               {
                                  "name":"write_debug",
                                  "type":"string",
                                  "value":"true",
                                  "fromParam":"write_debug"
                               },
                               {
                                  "name":"key",
                                  "type":"string",
                                  "value":"hello",
                                  "fromParam":"key"
                               }
                            ]))
                            .unwrap(),
                        ),
                        from_param: None,
                        required: false,
                        description: None,
                    }],
                    ..Default::default()
                },
                client: APIClient::new(Configuration {
                    base_path: ".".into(),
                    client: reqwest::Client::new(),
                }),
                params,
                owner_ref: None,
                annotations: None,
            },
        };
        let mut envs = BTreeMap::new();
        envs.insert("write_debug".to_string(), "false".to_string());
        envs.insert("key".to_string(), "hello".to_string());
        assert_eq!(of.extract_environment(), Some(envs))
    }
}
