use crate::schematic::traits::*;
use crate::workload_type::{REPLICATED_SERVICE_NAME, SINGLETON_NAME};

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

