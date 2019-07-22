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
fn test_autoscaler() {
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

    assert_eq!(
        Some(42),
        spec.metrics.expect("metrics")[0]
            .clone()
            .resource
            .expect("a resource")
            .target_average_utilization
    );
}
