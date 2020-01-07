use crate::schematic::traits::*;
use crate::workload_type::ParamMap;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use serde_json::json;
use std::collections::BTreeMap;
use serde_json::map::Map;

#[test]
fn test_ingress() {
    let mut params: ParamMap = BTreeMap::new();
    params.insert("service_port".into(), json!(8080));
    params.insert(
        "hostname".into(),
        serde_json::Value::String("in.example.com".into()),
    );
    params.insert("path".into(), json!("/path"));

    let ig = Ingress::from_params(
        "my-ingress".into(),
        "squid".into(),
        "patsy".into(),
        params,
        None,
    );

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
        "in.example.com",
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
        "/path",
        path.clone().path.expect("must be a path.path").as_str()
    );
    assert_eq!("squid", path.backend.service_name.as_str());
    assert_eq!(IntOrString::Int(8080), path.backend.service_port);
}

#[test]
fn test_ingress_service_port_as_string() {
    let mut params: ParamMap = BTreeMap::new();
    params.insert("service_port".into(), json!("8080"));
    params.insert(
        "hostname".into(),
        serde_json::Value::String("in.example.com".into()),
    );
    params.insert("path".into(), json!("/path"));

    let ig = Ingress::from_params(
        "my-ingress".into(),
        "squid".into(),
        "patsy".into(),
        params,
        None,
    );

    let king = ig.to_ext_ingress();
    let spec = king.spec.expect("spec is required");

    let rule = spec
        .rules
        .as_ref()
        .expect("rules are required")
        .get(0)
        .expect("a rule is required");

    let path = rule
        .http
        .as_ref()
        .expect("http is required")
        .paths
        .get(0)
        .expect("at least one path is required");
    assert_eq!(IntOrString::Int(8080), path.backend.service_port);
}

#[test]
fn test_ingress_invalid_service_port() {
    let mut params: ParamMap = BTreeMap::new();
    params.insert("service_port".into(), json!(8080.80));
    params.insert(
        "hostname".into(),
        serde_json::Value::String("in.example.com".into()),
    );
    params.insert("path".into(), json!("/path"));

    let ig = Ingress::from_params(
        "my-ingress".into(),
        "squid".into(),
        "patsy".into(),
        params,
        None,
    );

    let king = ig.to_ext_ingress();
    let spec = king.spec.expect("spec is required");

    let rule = spec
        .rules
        .as_ref()
        .expect("rules are required")
        .get(0)
        .expect("a rule is required");

    let path = rule
        .http
        .as_ref()
        .expect("http is required")
        .paths
        .get(0)
        .expect("at least one path is required");
    assert_eq!(IntOrString::Int(80), path.backend.service_port);
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
        owner_ref: None,
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
fn test_ingress_v1alpha1() {
    let ingress_alpha1_trait = TraitBinding {
        name : String::from("ingress.core.oam.dev/v1alpha1"),
        parameter_values: None,
        properties: Some(json!({
            "hostname": "in.example.com",
            "path": "/path",
            "servicePort": 9999
        }))
    };

    let serialized = serde_json::to_string(&ingress_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
    let prop_map : Option<&Map<String, serde_json::value::Value>> = deserialized_trait.properties.as_ref().unwrap().as_object();

    let ig = Ingress::from_properties(
        "my-ingress".into(),
        "squid".into(),
        "patsy".into(),
        prop_map,
        None,
    );

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
        "in.example.com",
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
        "/path",
        path.clone().path.expect("must be a path.path").as_str()
    );
    assert_eq!("squid", path.backend.service_name.as_str());
    assert_eq!(IntOrString::Int(9999), path.backend.service_port);
}

#[test]
fn test_ingress_v1alpha1_invalid_service_port() {
    let ingress_alpha1_trait = TraitBinding {
        name : String::from("ingress.core.oam.dev/v1alpha1"),
        parameter_values: None,
        properties: Some(json!({
            "hostname": "in.example.com",
            "path": "/path",
            "servicePort": 8080.01
        }))
    };

    let serialized = serde_json::to_string(&ingress_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
    let prop_map : Option<&Map<String, serde_json::value::Value>> = deserialized_trait.properties.as_ref().unwrap().as_object();
    let ig = Ingress::from_properties(
        "my-ingress".into(),
        "squid".into(),
        "patsy".into(),
        prop_map,
        None,
    );

    let king = ig.to_ext_ingress();
    let spec = king.spec.expect("spec is required");

    let rule = spec
        .rules
        .as_ref()
        .expect("rules are required")
        .get(0)
        .expect("a rule is required");

    let path = rule
        .http
        .as_ref()
        .expect("http is required")
        .paths
        .get(0)
        .expect("at least one path is required");
    assert_eq!(IntOrString::Int(80), path.backend.service_port);
}

#[test]
fn test_ingress_v1alpha_defaults() {

    let ingress_alpha1_trait = TraitBinding {
        name : String::from("ingress.core.oam.dev/v1alpha1"),
        parameter_values: None,
        properties: Some(json!({
            "servicePort": 9999
        }))
    };

    let serialized = serde_json::to_string(&ingress_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
    let prop_map : Option<&Map<String, serde_json::value::Value>> = deserialized_trait.properties.as_ref().unwrap().as_object();

    let ig = Ingress::from_properties(
        "my-ingress".into(),
        "squid".into(),
        "patsy".into(),
        prop_map,
        None,
    );

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
