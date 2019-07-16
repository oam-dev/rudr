use crate::schematic::traits::*;
use crate::workload_type::{REPLICATED_SERVICE_NAME, SINGLETON_NAME};
use kube::{client::APIClient, config::Configuration};

#[test]
fn test_ingress_workload_types() {
    assert!(Ingress::supports_workload_type(REPLICATED_SERVICE_NAME));
    assert!(Ingress::supports_workload_type(SINGLETON_NAME));
}

#[test]
fn test_autoscaler_workload_types() {
    assert!(Autoscaler::supports_workload_type(REPLICATED_SERVICE_NAME));
    assert!(!Autoscaler::supports_workload_type(SINGLETON_NAME));
}

#[test]
fn test_traits_exec() {
    let emptytrait = HydraTrait::Empty(Empty{});
    match emptytrait {
        HydraTrait::Empty(empty) => {
            assert!(empty.exec("test", mock_client(), Phase::Add).is_ok())
        },
        _ => panic!("Should be empty"),
    }
}

fn mock_client() -> APIClient {
    APIClient::new(
        Configuration {
            base_path: ".".into(),
            client: reqwest::Client::new(),
        }
    )
}

