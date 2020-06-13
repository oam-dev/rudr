use crate::schematic::configuration::ComponentConfiguration;
use crate::schematic::parameter::{
    self, extract_number_params, extract_string_params, ParameterValue,
};
use crate::schematic::scopes::HEALTH_SCOPE;
use failure::Error;
use kube::{api::CustomResource, client::Client};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use log::info;

pub const HEALTH_SCOPE_CRD: &str = "healthscopes";
pub const HEALTH_SCOPE_GROUP: &str = "core.oam.dev";
pub const HEALTH_SCOPE_VERSION: &str = "v1alpha1";
pub const HEALTH_SCOPE_KIND: &str = "HealthScope";
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HealthScope {
    pub probe_method: String,
    pub probe_endpoint: String,
    pub probe_timeout: Option<i64>,
    pub probe_interval: Option<i64>,
    pub failure_rate_threshold: Option<f64>,
    pub healthy_rate_threshold: Option<f64>,
    pub health_threshold_percentage: Option<f64>,
    pub required_healthy_components: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInfo {
    pub name: String,
    pub instance_name: String,
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub components: Option<Vec<ComponentInfo>>,
    pub last_aggregate_timestamp: Option<String>,
}
impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus {
            components: None,
            last_aggregate_timestamp: None,
        }
    }
}

pub type HealthScopeObject = kube::api::Object<HealthScope, HealthStatus>;

/// Health scope is defined as https://github.com/oam-dev/spec/blob/master/4.application_scopes.md#health-scope
#[derive(Clone)]
pub struct Health {
    client: Client,
    namespace: String,
    pub name: String,
    pub allow_component_overlap: bool,
    pub probe_method: String,
    pub probe_endpoint: String,
    pub probe_timeout: Option<i64>,
    pub probe_interval: Option<i64>,
    pub failure_rate_threshold: Option<f64>,
    pub healthy_rate_threshold: Option<f64>,
    pub health_threshold_percentage: Option<f64>,
    pub required_healthy_components: Option<Vec<String>>,
}

impl Health {
    pub fn from_params(
        name: String,
        namespace: String,
        client: Client,
        params: Vec<ParameterValue>,
    ) -> Result<Self, Error> {
        let probe_method = match extract_string_params("probe-method", params.clone()) {
            Some(probe_method) => probe_method,
            None => {
                return Err(format_err!(
                    "probe-method does not exist in params {:?}",
                    params
                ))
            }
        };
        let probe_endpoint = match extract_string_params("probe-endpoint", params.clone()) {
            Some(probe_endpoint) => probe_endpoint,
            None => {
                return Err(format_err!(
                    "probe-endpoint does not exist in params {:?}",
                    params
                ))
            }
        };
        let probe_timeout =
            extract_number_params("probe-timeout", params.clone()).and_then(|v| v.as_i64());
        let probe_interval =
            extract_number_params("probe-interval", params.clone()).and_then(|v| v.as_i64());
        let failure_rate_threshold =
            extract_number_params("failure-rate-threshold", params.clone())
                .and_then(|v| v.as_f64());
        let healthy_rate_threshold =
            extract_number_params("healthy-rate-threshold", params.clone())
                .and_then(|v| v.as_f64());
        let health_threshold_percentage =
            extract_number_params("health-threshold-percentage", params.clone())
                .and_then(|v| v.as_f64());
        let required_healthy_components =
            parameter::extract_value_params("required-healthy-components", params.clone())
                .and_then(|v| v.as_array().cloned())
                .and_then(|v| {
                    v.iter()
                        .map(|x| x.as_str().and_then(|v| Some(v.to_string())))
                        .clone()
                        .collect()
                });
        Ok(Health {
            name,
            namespace,
            client,
            allow_component_overlap: true,
            probe_method,
            probe_endpoint,
            probe_timeout,
            probe_interval,
            failure_rate_threshold,
            healthy_rate_threshold,
            health_threshold_percentage,
            required_healthy_components,
        })
    }
    pub fn allow_overlap(&self) -> bool {
        self.allow_component_overlap
    }
    pub fn scope_type(&self) -> String {
        String::from(HEALTH_SCOPE)
    }
    pub async fn create(&self, owner: OwnerReference) -> Result<(), Error> {
        let pp = kube::api::PostParams::default();
        let mut owners = vec![];
        owners.insert(0, owner);
        let scope = HealthScopeObject {
            spec: HealthScope {
                probe_method: self.probe_method.clone(),
                probe_endpoint: self.probe_endpoint.clone(),
                probe_timeout: self.probe_timeout,
                probe_interval: self.probe_interval,
                failure_rate_threshold: self.failure_rate_threshold,
                healthy_rate_threshold: self.healthy_rate_threshold,
                health_threshold_percentage: self.health_threshold_percentage,
                required_healthy_components: self.required_healthy_components.clone(),
            },
            types: kube::api::TypeMeta {
                api_version: HEALTH_SCOPE_GROUP.to_string() + "/" + HEALTH_SCOPE_VERSION,
                kind: HEALTH_SCOPE_KIND.to_string(),
            },
            metadata: kube::api::ObjectMeta {
                name: Some(self.name.clone()),
                owner_references: Some(owners),
                ..Default::default()
            },
            status: None,
        };
        let healthscope_resource = CustomResource::kind(HEALTH_SCOPE_CRD)
            .version(HEALTH_SCOPE_VERSION)
            .group(HEALTH_SCOPE_GROUP)
            .within(self.namespace.as_str())
            .into_resource();
        let req = healthscope_resource.create(&pp, serde_json::to_vec(&scope)?)?;
        let err = self
            .client
            .request::<HealthScopeObject>(req)
            .await
            .err()
            .and_then(|e| {
                let api_err = match e {
                    kube::Error::Api(ref err) => Some(err),
                    _ => None
                };
                let exist = api_err
                    .and_then(|api_err| {
                        if api_err.reason.eq("AlreadyExists") {
                            return Some(());
                        }
                        None
                    })
                    .is_some();
                if exist {
                    return None;
                }
                Some(e)
            });
        if let Some(e) = err {
            return Err(e.into());
        }
        info!("health scope {} created", self.name.clone());
        Ok(())
    }
    pub async fn modify(&self) -> Result<(), Error> {
        Err(format_err!("health scope modify not implemented"))
    }
    /// let OwnerReference delete
    pub async fn delete(&self) -> Result<(), Error> {
        Ok(())
    }
    pub async fn add(&self, spec: ComponentConfiguration) -> Result<(), Error> {
        let mut obj = self.get_obj().await?;
        let mut components = self.remove_one(spec.clone(), obj.status.clone());
        components.insert(
            components.len(),
            ComponentInfo {
                name: spec.component_name.clone(),
                instance_name: spec.instance_name.clone(),
                status: None,
            },
        );
        obj.status = Some(HealthStatus {
            components: Some(components),
            ..Default::default()
        });
        info!(
            "add component {} to health scope {}",
            spec.component_name.clone(),
            self.name.clone()
        );
        self.patch_obj(obj).await
    }
    pub async fn remove(&self, spec: ComponentConfiguration) -> Result<(), Error> {
        let mut obj = self.get_obj().await?;
        let components = self.remove_one(spec.clone(), obj.status.clone());
        obj.status = Some(HealthStatus {
            components: Some(components),
            ..Default::default()
        });
        self.patch_obj(obj).await
    }

    pub async fn get_obj(&self) -> Result<HealthScopeObject, Error> {
        let healthscope_resource = CustomResource::kind(HEALTH_SCOPE_CRD)
            .version(HEALTH_SCOPE_VERSION)
            .group(HEALTH_SCOPE_GROUP)
            .within(self.namespace.as_str())
            .into_resource();
        let req = healthscope_resource.get(self.name.as_str())?;
        Ok(self.client.request::<HealthScopeObject>(req).await?)
    }
    fn remove_one(
        &self,
        spec: ComponentConfiguration,
        status: Option<HealthStatus>,
    ) -> Vec<ComponentInfo> {
        let mut components = vec![];
        if let Some(status) = status {
            for comp in status.components.unwrap_or_else(|| vec![]).iter() {
                if comp.name == spec.component_name && comp.instance_name == spec.instance_name {
                    continue;
                }
                components.insert(components.len(), comp.clone())
            }
        }
        components
    }
    async fn patch_obj(&self, obj: HealthScopeObject) -> Result<(), Error> {
        let pp = kube::api::PatchParams::default();
        let healthscope_resource = CustomResource::kind(HEALTH_SCOPE_CRD)
            .version(HEALTH_SCOPE_VERSION)
            .group(HEALTH_SCOPE_GROUP)
            .within(self.namespace.as_str())
            .into_resource();
        let req = healthscope_resource.patch(self.name.as_str(), &pp, serde_json::to_vec(&obj)?)?;
        self.client.request::<HealthScopeObject>(req).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::schematic::parameter::ParameterValue;
    use crate::schematic::scopes::{health::Health, HEALTH_SCOPE};
    use kube::client::Client;
    use kube::config::Config;
    /// This mock builds a KubeConfig that will not be able to make any requests.
    fn mock_kube_config() -> Config {
        Config::new("http://localhost".parse().unwrap())
    }
    #[test]
    fn test_create_health() {
        let mut params = vec![];
        params.insert(
            params.len(),
            ParameterValue {
                name: "probe-method".to_string(),
                value: Some("httpGet".into()),
                from_param: None,
            },
        );
        params.insert(
            params.len(),
            ParameterValue {
                name: "probe-endpoint".to_string(),
                value: Some("/v1/health".into()),
                from_param: None,
            },
        );
        params.insert(
            params.len(),
            ParameterValue {
                name: "probe-timeout".to_string(),
                value: Some(10.into()),
                from_param: None,
            },
        );
        params.insert(
            params.len(),
            ParameterValue {
                name: "failure-rate-threshold".to_string(),
                value: Some(80.into()),
                from_param: None,
            },
        );
        let mut comps = vec![];
        comps.insert(0, serde_json::Value::from("comp1"));
        comps.insert(1, serde_json::Value::from("comp2"));
        params.insert(
            params.len(),
            ParameterValue {
                name: "required-healthy-components".to_string(),
                value: Some(serde_json::Value::Array(comps)),
                from_param: None,
            },
        );

        let net = Health::from_params(
            "test-health".to_string(),
            "namespace".to_string(),
            Client::new(mock_kube_config()),
            params,
        )
        .unwrap();
        assert_eq!(true, net.allow_overlap());
        assert_eq!(HEALTH_SCOPE.to_string(), net.scope_type());
        assert_eq!("test-health".to_string(), net.name);
        assert_eq!("httpGet".to_string(), net.probe_method);
        assert_eq!("/v1/health".to_string(), net.probe_endpoint);
        assert_eq!(Some(10), net.probe_timeout);
        assert_eq!(Some(80.0), net.failure_rate_threshold);
        let mut comps = vec![];
        comps.insert(0, "comp1".to_string());
        comps.insert(1, "comp2".to_string());
        assert_eq!(Some(comps), net.required_healthy_components);
    }
}
