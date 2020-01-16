use k8s_openapi::api::{apps::v1 as apps, batch::v1 as batch};
use crate::{
        schematic::traits::*,
        workload_type::{SERVER_NAME, SINGLETON_SERVER_NAME, SINGLETON_TASK_NAME, TASK_NAME},
};
use serde_json::json;
use serde_json::map::Map;

#[test]
fn test_manual_scaler_workload_types() {
    let matches = vec![SERVER_NAME, TASK_NAME];
    for m in matches {
        assert!(ManualScaler::supports_workload_type(m));
    }
    let no_matches = vec![SINGLETON_TASK_NAME, SINGLETON_SERVER_NAME];
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
    let ms = ManualScaler {
        name: "name".into(),
        instance_name: "inst_name".into(),
        component_name: "comp_name".into(),
        owner_ref: None,
        replica_count: 9,
        workload_type: SERVER_NAME.into(),
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
    let ms = ManualScaler {
        name: "name".into(),
        instance_name: "inst_name".into(),
        component_name: "comp_name".into(),
        owner_ref: None,
        replica_count: 9,
        workload_type: TASK_NAME.into(),
    };
    let second = ms.scale_job(first);
    assert_eq!(Some(9), second.spec.expect("spec is required").parallelism);
}

#[test]
fn test_manual_scaler_v1alpha1_properties() {
    let first = batch::Job {
        spec: Some(batch::JobSpec {
            parallelism: Some(3),
            ..Default::default()
        }),
        ..Default::default()
    };

    let manualscaler_alpha1_trait = TraitBinding {
        name : String::from("manual-scaler"),
		parameter_values: None,
        properties: Some(json!({
		    "replicaCount": 3
        }))
    };

	let serialized = serde_json::to_string(&manualscaler_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
	let prop_map : Option<&Map<String, serde_json::value::Value>> = deserialized_trait.properties.as_ref().unwrap().as_object();

    let ms = ManualScaler::from_properties(
        "release".into(),
        "instance".into(),
        "component".into(),
        prop_map,
        None,
		"core.oam.dev/v1alpha1.Task".into(),
    );

    let second = ms.scale_job(first);
    assert_eq!(Some(3), second.spec.expect("spec is required").parallelism);
}