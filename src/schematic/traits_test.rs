use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;

use crate::schematic::traits::*;
use crate::workload_type::{ParamMap, REPLICATED_SERVICE_NAME, SINGLETON_NAME};
use std::collections::BTreeMap;

#[test]
fn test_ingress() {
    let mut params: ParamMap = BTreeMap::new();
    params.insert("service_port".into(), json!(8080));
    params.insert(
        "hostname".into(),
        serde_json::Value::String("in.example.com".into()),
    );
    params.insert("path".into(), json!("/path"));

    let ig = Ingress::from_params("my-ingress".into(), "squid".into(), "patsy".into(), params);

    let king = ig.to_ext_ingress();
    assert_eq!(
        "squid-trait-ingress",
        king.metadata
            .expect("md must exits")
            .name
            .expect("name must exist")
    );

    let spec = king.spec.expect("spec is required");
    assert_eq!(1, spec.rules.as_ref().unwrap().len());

    let rule = spec
        .rules
        .as_ref()
        .expect("rules are required")
        .get(0)
        .expect("a rule is required");
    assert_eq!(
        "\"in.example.com\"",
        rule.host.as_ref().expect("host is required").to_string()
    );

    let path = rule
        .http
        .as_ref()
        .expect("http is required")
        .paths
        .get(0)
        .expect("at least one path is required");
    assert_eq!(
        "\"/path\"",
        path.clone().path.expect("must be a path.path").as_str()
    );
    assert_eq!("my-ingress", path.backend.service_name.as_str());
    assert_eq!(IntOrString::Int(8080), path.backend.service_port);
}

#[test]
fn test_ingress_defaults() {
    let ig = Ingress {
        name: "my-ingress".into(),
        instance_name: "squid".into(),
        component_name: "patsy".into(),
        svc_port: 8080,
        hostname: None,
        path: None,
    };

    let king = ig.to_ext_ingress();
    let rule = king
        .spec
        .as_ref()
        .expect("spec required")
        .rules
        .as_ref()
        .expect("rules required")
        .get(0)
        .expect("one rule required");
    assert_eq!(Some("example.com".into()), rule.host);
    assert_eq!(
        Some("/".into()),
        rule.http
            .as_ref()
            .expect("http required")
            .paths
            .get(0)
            .expect("path required")
            .path
    );
}

#[test]
fn test_autoscaler_defaults() {
    let autoscaler = Autoscaler {
        name: "release".into(),
        instance_name: "instance".into(),
        component_name: "component".into(),
        cpu: None,
        minimum: None,
        maximum: None,
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

    let autoscaler = Autoscaler::from_params("release".into(), "instance".into(), "component".into(), params);
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
fn test_trait_labels() {
    let labels = trait_labels();
    assert_eq!(
        "trait".to_string(),
        *labels.get("hydra.io/role").expect("must be a string")
    );
}
