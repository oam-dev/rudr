use crate::schematic::traits::*;
use crate::workload_type::ParamMap;
use std::collections::BTreeMap;

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

    let metrics = spec.metrics.expect("metrics").clone();
    assert_eq!(
        Some(50),
        metrics[0]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );

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

    let metrics = spec.metrics.expect("metrics").clone();
    assert_eq!(
        Some(42),
        metrics[0]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );

    assert_eq!(
        Some(50),
        metrics[1]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );
}


