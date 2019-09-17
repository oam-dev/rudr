/// Network scope is defined as https://github.com/microsoft/hydra-spec/blob/master/4.application_scopes.md#network-scope
/// Now we don't really implement network scope, this is just a framework as the spec describe.
use crate::schematic::parameter::{extract_string_params, ParameterValue};
use crate::schematic::scopes::NETWORK_SCOPE;
use failure::{format_err, Error};
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

#[derive(Clone)]
pub struct Network {
    client: APIClient,
    pub name: String,
    pub allow_component_overlap: bool,
    pub network_id: String,
    pub subnet_id: String,
    pub internet_gateway_type: Option<String>,
}

impl Network {
    pub fn from_params(
        name: String,
        client: APIClient,
        params: Vec<ParameterValue>,
    ) -> Result<Self, Error> {
        let network_id = match extract_string_params("network-id", params.clone()) {
            Some(network_id) => network_id,
            None => return Err(format_err!("network-id is not exist")),
        };
        let subnet_id = match extract_string_params("subnet-id", params.clone()) {
            Some(network_id) => network_id,
            None => return Err(format_err!("subnet-id is not exist")),
        };
        Ok(Network {
            network_id,
            subnet_id,
            name,
            client,
            internet_gateway_type: extract_string_params("internet-gateway-type", params.clone()),
            allow_component_overlap: false,
        })
    }
    pub fn allow_overlap(&self) -> bool {
        self.allow_component_overlap
    }
    pub fn scope_type(&self) -> String {
        String::from(NETWORK_SCOPE)
    }
    pub fn create(&self, _ns: &str, _owner: meta::OwnerReference) -> Result<(), Error> {
        Err(format_err!("network scope create not implemented"))
    }
    pub fn modify(&self, _ns: &str) -> Result<(), Error> {
        Err(format_err!("network scope modify not implemented"))
    }
    pub fn delete(&self, _ns: &str) -> Result<(), Error> {
        Err(format_err!("network scope delete not implemented"))
    }
}

#[cfg(test)]
mod test {
    use crate::schematic::parameter::ParameterValue;
    use crate::schematic::scopes::{Network, NETWORK_SCOPE};
    use kube::client::APIClient;
    use kube::config::Configuration;
    /// This mock builds a KubeConfig that will not be able to make any requests.
    fn mock_kube_config() -> Configuration {
        Configuration {
            base_path: ".".into(),
            client: reqwest::Client::new(),
        }
    }

    #[test]
    fn test_create_network() {
        let mut params = vec![];
        params.insert(
            0,
            ParameterValue {
                name: "network-id".to_string(),
                value: Some("nid".into()),
                from_param: None,
            },
        );
        params.insert(
            1,
            ParameterValue {
                name: "subnet-id".to_string(),
                value: Some("sid".into()),
                from_param: None,
            },
        );
        let net = Network::from_params(
            "test-net".to_string(),
            APIClient::new(mock_kube_config()),
            params,
        )
        .unwrap();
        assert_eq!(false, net.allow_overlap());
        assert_eq!(NETWORK_SCOPE.to_string(), net.scope_type());
        assert_eq!("nid".to_string(), net.network_id);
        assert_eq!("sid".to_string(), net.subnet_id);
    }
}
