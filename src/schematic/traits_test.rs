use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;

use crate::schematic::traits;

#[test]
fn test_ingress() {
    let ig = traits::Ingress {
        name: "my-ingress".into(),
        component_name: "patsy".into(),
        svc_port: 8080,
        hostname: Some("in.example.com".to_string()),
        path: Some("/path".to_string()),
    };

    let king = ig.to_ext_ingress();
    assert_eq!(
        "my-ingress-patsy-trait-ingress",
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
        rule.host.as_ref().expect("host is required").as_str()
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
    assert_eq!("my-ingress", path.backend.service_name.as_str());
    assert_eq!(IntOrString::Int(8080), path.backend.service_port);
}

#[test]
fn test_ingress_defaults() {
    let ig = traits::Ingress {
        name: "my-ingress".into(),
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
