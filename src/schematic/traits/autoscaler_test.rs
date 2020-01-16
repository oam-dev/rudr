use crate::schematic::traits::*;
use crate::workload_type::ParamMap;
use serde_json::json;
use std::collections::BTreeMap;
use serde_json::map::Map;

#[test]
fn test_autoscaler_defaults() {
    let autoscaler = Autoscaler {
        name: "release".into(),
        instance_name: "instance".into(),
        component_name: "component".into(),
        cpu: None,
        memory: None,
        minimum: None,
        maximum: None,
        owner_ref: None,
    };
    let kauto = autoscaler.to_horizontal_pod_autoscaler();
    assert_eq!(
        Some("instance-trait-autoscaler".to_string()),
        kauto.metadata.expect("metadata").name
    );
    let spec = kauto.spec.expect("spec");
    assert_eq!(10, spec.max_replicas);
}

#[test]
fn test_autoscaler_cpu() {
    let mut params: ParamMap = BTreeMap::new();
    params.insert("cpu".into(), json!(42));
    params.insert("minimum".into(), json!(6));
    params.insert("maximum".into(), json!(7));

    let autoscaler = Autoscaler::from_params(
        "release".into(),
        "instance".into(),
        "component".into(),
        params,
        None,
    );
    let kauto = autoscaler.to_horizontal_pod_autoscaler();
    assert_eq!(
        Some("instance-trait-autoscaler".to_string()),
        kauto.metadata.expect("metadata").name
    );
    let spec = kauto.spec.expect("spec");
    assert_eq!(7, spec.max_replicas);
    assert_eq!(Some(6), spec.min_replicas);

    // cpu is added first so index for cpu is 0
    let metrics = spec.metrics.expect("metrics").clone();
    assert_eq!(
        Some(42),
        metrics[0]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );
}

#[test]
fn test_autoscaler_memory() {
    let mut params: ParamMap = BTreeMap::new();
    params.insert("memory".into(), json!(50));
    params.insert("minimum".into(), json!(6));
    params.insert("maximum".into(), json!(7));

    let autoscaler = Autoscaler::from_params(
        "release".into(),
        "instance".into(),
        "component".into(),
        params,
        None,
    );
    let kauto = autoscaler.to_horizontal_pod_autoscaler();
    assert_eq!(
        Some("instance-trait-autoscaler".to_string()),
        kauto.metadata.expect("metadata").name
    );
    let spec = kauto.spec.expect("spec");
    assert_eq!(7, spec.max_replicas);
    assert_eq!(Some(6), spec.min_replicas);

    // cpu is added first so index for memory is 1
    let metrics = spec.metrics.expect("metrics").clone();
    assert_eq!(
        Some(50),
        metrics[0]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );
}

#[test]
fn test_autoscaler_multi_metrics_resource() {
    let mut params: ParamMap = BTreeMap::new();
    params.insert("cpu".into(), json!(42));
    params.insert("memory".into(), json!(50));
    params.insert("minimum".into(), json!(6));
    params.insert("maximum".into(), json!(7));

    let autoscaler = Autoscaler::from_params(
        "release".into(),
        "instance".into(),
        "component".into(),
        params,
        None,
    );
    let kauto = autoscaler.to_horizontal_pod_autoscaler();
    assert_eq!(
        Some("instance-trait-autoscaler".to_string()),
        kauto.metadata.expect("metadata").name
    );
    let spec = kauto.spec.expect("spec");
    assert_eq!(7, spec.max_replicas);
    assert_eq!(Some(6), spec.min_replicas);

    // cpu is added first so index for cpu is 0
    let metrics = spec.metrics.expect("metrics").clone();
    assert_eq!(
        Some(42),
        metrics[0]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );

    // cpu is added first so index for memory is 1
    assert_eq!(
        Some(50),
        metrics[1]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );
}

#[test]
fn test_autoscaler_v1alpha1_cpu() {
    let autoscaler_alpha1_trait = TraitBinding {
        name : String::from("auto-scaler"),
        parameter_values: None,
        properties: Some(json!({
            "cpu": 42,
            "minimum": 6,
            "maximum": 7
        }))
    };

    let serialized = serde_json::to_string(&autoscaler_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
    let prop_map : Option<&Map<String, serde_json::value::Value>> = deserialized_trait.properties.as_ref().unwrap().as_object();

    let autoscaler = Autoscaler::from_properties(
        "release".into(),
        "instance".into(),
        "component".into(),
        prop_map,
        None,
    );
    let kauto = autoscaler.to_horizontal_pod_autoscaler();
    assert_eq!(
        Some("instance-trait-autoscaler".to_string()),
        kauto.metadata.expect("metadata").name
    );
    let spec = kauto.spec.expect("spec");
    assert_eq!(7, spec.max_replicas);
    assert_eq!(Some(6), spec.min_replicas);

    // cpu is added first so index for cpu is 0
    let metrics = spec.metrics.expect("metrics").clone();
    assert_eq!(
        Some(42),
        metrics[0]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );
}

#[test]
fn test_autoscaler_v1alpha1_memory() {
    let autoscaler_alpha1_trait = TraitBinding {
        name : String::from("auto-scaler"),
        parameter_values: None,
        properties: Some(json!({
            "memory": 50,
            "minimum": 6,
            "maximum": 7
        }))
    };

    let serialized = serde_json::to_string(&autoscaler_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
    let prop_map : Option<&Map<String, serde_json::value::Value>> = deserialized_trait.properties.as_ref().unwrap().as_object();

    let autoscaler = Autoscaler::from_properties(
        "release".into(),
        "instance".into(),
        "component".into(),
        prop_map,
        None,
    );
    let kauto = autoscaler.to_horizontal_pod_autoscaler();
    assert_eq!(
        Some("instance-trait-autoscaler".to_string()),
        kauto.metadata.expect("metadata").name
    );
    let spec = kauto.spec.expect("spec");
    assert_eq!(7, spec.max_replicas);
    assert_eq!(Some(6), spec.min_replicas);

    // cpu is added first so index for memory is 1
    let metrics = spec.metrics.expect("metrics").clone();
    assert_eq!(
        Some(50),
        metrics[0]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );
}

#[test]
fn test_autoscaler_v1alpha1_multi_metrics_resource() {
    let autoscaler_alpha1_trait = TraitBinding {
        name : String::from("auto-scaler"),
        parameter_values: None,
        properties: Some(json!({
            "cpu": 42,
            "memory": 50,
            "minimum": 6,
            "maximum": 7
        }))
    };

    let serialized = serde_json::to_string(&autoscaler_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
    let prop_map : Option<&Map<String, serde_json::value::Value>> = deserialized_trait.properties.as_ref().unwrap().as_object();

    let autoscaler = Autoscaler::from_properties(
        "release".into(),
        "instance".into(),
        "component".into(),
        prop_map,
        None,
    );
    let kauto = autoscaler.to_horizontal_pod_autoscaler();
    assert_eq!(
        Some("instance-trait-autoscaler".to_string()),
        kauto.metadata.expect("metadata").name
    );
    let spec = kauto.spec.expect("spec");
    assert_eq!(7, spec.max_replicas);
    assert_eq!(Some(6), spec.min_replicas);

    // cpu is added first so index for cpu is 0
    let metrics = spec.metrics.expect("metrics").clone();
    assert_eq!(
        Some(42),
        metrics[0]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );

    // cpu is added first so index for memory is 1
    assert_eq!(
        Some(50),
        metrics[1]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );
}
