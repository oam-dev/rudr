use k8s_openapi::api::{
    apps::v1 as apps,
    batch::v1 as batch,
};

use crate::{
    schematic::traits::{
        TraitImplementation,
        manual_scaler::ManualScaler,
    },
    workload_type::{REPLICATED_SERVICE_NAME, REPLICATED_TASK_NAME, TASK_NAME, SINGLETON_NAME},
};

#[test]
fn test_manual_scaler_workload_types() {
    let matches = vec![REPLICATED_SERVICE_NAME, REPLICATED_TASK_NAME];
    for m in matches {
        assert!(ManualScaler::supports_workload_type(m));
    }
    let no_matches = vec![TASK_NAME, SINGLETON_NAME];
    for m in no_matches {
        assert!(!ManualScaler::supports_workload_type(m));
    }
}

#[test]
fn test_scale_deployment() {
    let first = apps::Deployment {
        spec: Some(apps::DeploymentSpec {
            replicas: Some(5),
            ..Default::default()
        }),
        ..Default::default()
    };
    let ms = ManualScaler{
        name: "name".into(),
        instance_name: "inst_name".into(),
        component_name: "comp_name".into(),
        owner_ref: None,
        replica_count: 9,
        workload_type: REPLICATED_SERVICE_NAME.into(),
    };
    let second = ms.scale_deployment(first);
    assert_eq!(Some(9), second.spec.expect("spec is required").replicas);
}

#[test]
fn test_scale_job() {
    let first = batch::Job {
        spec: Some(batch::JobSpec {
            parallelism: Some(5),
            ..Default::default()
        }),
        ..Default::default()
    };
    let ms = ManualScaler{
        name: "name".into(),
        instance_name: "inst_name".into(),
        component_name: "comp_name".into(),
        owner_ref: None,
        replica_count: 9,
        workload_type: REPLICATED_TASK_NAME.into(),
    };
    let second = ms.scale_job(first);
    assert_eq!(Some(9), second.spec.expect("spec is required").parallelism);
}