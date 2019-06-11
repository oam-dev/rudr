use crate::schematic::parameter::*;

#[test]
fn test_resolve_parameters() {
    let params = vec![
        Parameter {
            name: "email".into(),
            description: Some("a valid email address".into()),
            parameter_type: ParameterType::String,
            required: true,
            default: None,
        },
        Parameter {
            name: "yob".into(),
            description: Some("year of birth".into()),
            parameter_type: ParameterType::Number,
            default: Some(json!(1912)),
            required: false,
        },
    ];
    let vals1 = vec![
        ParameterValue {
            name: "email".into(),
            value: json!("eliot@example.com"),
        },
        ParameterValue {
            name: "yob".into(),
            value: json!(1888),
        },
    ];

    let res = resolve_parameters(params.clone(), vals1).expect("vals1 should pass successfully");
    assert_eq!(
        json!("eliot@example.com"),
        *res.get("email").expect("email should be set")
    );
    assert_eq!(json!(1888), *res.get("yob").expect("yob should be set"));

    // This set of values should fail. Email is required.
    let vals2 = vec![ParameterValue {
        name: "yob".into(),
        value: json!(1888),
    }];

    let res = resolve_parameters(params.clone(), vals2);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("validation failed"));

    // Type mismatch
    let vals3 = vec![
        ParameterValue {
            name: "email".into(),
            value: json!("eliot@example.com"),
        },
        ParameterValue {
            name: "yob".into(),
            value: json!("I'm a string"),
        },
    ];
    let res = resolve_parameters(params.clone(), vals3);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("validation failed"));

    // Check that default field works.
    let vals3 = vec![ParameterValue {
        name: "email".into(),
        value: json!("eliot@example.com"),
    }];
    let res = resolve_parameters(params.clone(), vals3).expect("default value is okay");
    assert_eq!(
        json!(1912),
        *res.get("yob").expect("yob should be set to default")
    );
}
