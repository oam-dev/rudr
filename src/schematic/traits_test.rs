use crate::lifecycle::Phase;
use crate::schematic::traits::*;
use crate::workload_type::{SERVER_NAME, SINGLETON_SERVER_NAME};
use kube::{client::APIClient, config::Configuration};

#[test]
fn test_ingress_workload_types() {
    assert!(Ingress::supports_workload_type(SERVER_NAME));
    assert!(Ingress::supports_workload_type(SINGLETON_SERVER_NAME));
}

#[test]
fn test_autoscaler_workload_types() {
    assert!(Autoscaler::supports_workload_type(SERVER_NAME));
    assert!(!Autoscaler::supports_workload_type(SINGLETON_SERVER_NAME));
}

#[test]
fn test_traits_exec() {
    let emptytrait = OAMTrait::Empty(Empty {});
    match emptytrait {
        OAMTrait::Empty(empty) => assert!(empty.exec("test", mock_client(), Phase::Add).is_ok()),
        _ => panic!("Should be empty"),
    }
}

fn mock_client() -> APIClient {
    APIClient::new(Configuration {
        base_path: ".".into(),
        client: reqwest::Client::new(),
    })
}
