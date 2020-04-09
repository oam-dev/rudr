use crate::schematic::traits::*;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use serde_json::json;
use serde_json::map::Map;

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
        tls_hosts: None,
        tls_secret_name: None,
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
    
    let tls_vec = king.spec
        .as_ref()
        .expect("spec required")
        .tls
        .as_ref()
        .expect("tls expected");
    assert_eq!(0, tls_vec.len());
}

#[test]
fn test_ingress_v1alpha1() {
    let ingress_alpha1_trait = TraitBinding {
        name: String::from("ingress"),
        parameter_values: None,
        properties: Some(json!({
            "hostname": "in.example.com",
            "path": "/path",
            "servicePort": 9999,
            "tlsHosts": "x.example.com,y.example.com,z.example.com",
            "tlsSecretName": "my_secret",
        })),
    };

    let serialized = serde_json::to_string(&ingress_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
    let prop_map: Option<&Map<String, serde_json::value::Value>> =
        deserialized_trait.properties.as_ref().unwrap().as_object();

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

    let tls = spec
        .tls
        .as_ref()
        .expect("tls expected")
        .get(0)
        .expect("one tls expected");
    assert_eq!("my_secret", tls.secret_name.as_ref().expect("secret expected"));
    let tls_hosts = tls.hosts.as_ref().expect("hosts are expected");
    assert_eq!(3, tls_hosts.len());
    assert_eq!("x.example.com", tls_hosts.get(0).expect("host is expected"));
    assert_eq!("y.example.com", tls_hosts.get(1).expect("host is expected"));
    assert_eq!("z.example.com", tls_hosts.get(2).expect("host is expected"));
}

#[test]
fn test_ingress_v1alpha1_invalid_service_port() {
    let ingress_alpha1_trait = TraitBinding {
        name: String::from("ingress"),
        parameter_values: None,
        properties: Some(json!({
            "hostname": "in.example.com",
            "path": "/path",
            "servicePort": 8080.01
        })),
    };

    let serialized = serde_json::to_string(&ingress_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
    let prop_map: Option<&Map<String, serde_json::value::Value>> =
        deserialized_trait.properties.as_ref().unwrap().as_object();
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
        name: String::from("ingress"),
        parameter_values: None,
        properties: Some(json!({
            "servicePort": 9999
        })),
    };

    let serialized = serde_json::to_string(&ingress_alpha1_trait).unwrap();
    let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
    let prop_map: Option<&Map<String, serde_json::value::Value>> =
        deserialized_trait.properties.as_ref().unwrap().as_object();

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
