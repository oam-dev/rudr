use crate::schematic::parameter::*;
use serde_json::json;
use std::collections::BTreeMap;

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
    let mut vals1: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    vals1.insert("email".into(), json!("eliot@example.com"));
    vals1.insert("yob".into(), json!(1888));

    let res = resolve_parameters(params.clone(), vals1).expect("vals1 should pass successfully");
    assert_eq!(
        json!("eliot@example.com"),
        *res.get("email").expect("email should be set")
    );
    assert_eq!(json!(1888), *res.get("yob").expect("yob should be set"));

    // This set of values should fail. Email is required.
    let mut vals2 = BTreeMap::new();
    vals2.insert("yob".into(), json!(1888));

    let res = resolve_parameters(params.clone(), vals2);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("validation failed"));

    // Type mismatch
    let mut vals3 = BTreeMap::new();
    vals3.insert("email".into(), json!("eliot@example.com"));
    vals3.insert("yob".into(), json!("not an integer"));
    let res = resolve_parameters(params.clone(), vals3);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("validation failed"));

    // Check that default field works.
    let mut vals3 = BTreeMap::new();
    vals3.insert("email".into(), json!("eliot@example.com"));
    let res = resolve_parameters(params.clone(), vals3).expect("default value is okay");
    assert_eq!(
        json!(1912),
        *res.get("yob").expect("yob should be set to default")
    );
}

#[test]
fn test_resolve_values() {
    let parent = vec![
        ParameterValue {
            name: "pet".into(),
            value: Some(json!("dog")),
            from_param: None,
        },
        ParameterValue {
            name: "home".into(),
            value: Some(json!("house")),
            from_param: None,
        },
    ];
    let child = vec![
        ParameterValue {
            name: "favorite_animal".into(),
            value: None,
            from_param: Some("pet".to_string()),
        },
        ParameterValue {
            name: "abode".into(),
            value: None,
            from_param: Some("home".to_string()),
        },
    ];

    let merged = resolve_values(child, parent).expect("resolve parent values into child");
    assert_eq!(
        Some("house"),
        merged.get("abode").expect("abode is home").as_str()
    );
    assert_eq!(
        Some("dog"),
        merged
            .get("favorite_animal")
            .expect("abode is home")
            .as_str()
    );

    // Failed `from`
    let parent = vec![ParameterValue {
        name: "home".into(),
        value: Some(json!("house")),
        from_param: None,
    }];
    let child = vec![
        ParameterValue {
            name: "favorite_animal".into(),
            value: None,
            from_param: Some("pet".to_string()),
        },
        ParameterValue {
            name: "abode".into(),
            value: None,
            from_param: Some("home".to_string()),
        },
    ];

    assert_eq!(
        "could not resolve fromParam:pet for favorite_animal",
        resolve_values(child, parent)
            .expect_err("expected failure")
            .to_string()
    );

    // No from, but with a default value.
    let parent = vec![ParameterValue {
        name: "home".into(),
        value: Some(json!("house")),
        from_param: None,
    }];
    let child = vec![
        ParameterValue {
            name: "favorite_animal".into(),
            value: Some(json!("cat")),
            from_param: Some("pet".to_string()),
        },
        ParameterValue {
            name: "abode".into(),
            value: Some(json!("condo")),
            from_param: Some("home".to_string()),
        },
    ];

    let merged = resolve_values(child, parent).expect("should parse fine");
    assert_eq!(
        Some("cat"),
        merged
            .get("favorite_animal")
            .expect("favorite animal")
            .as_str()
    );
    assert_eq!(Some("house"), merged.get("abode").expect("abode").as_str());
}

#[test]
fn test_resolve_value() {
    let mut params: ResolvedVals = BTreeMap::new();
    params.insert("abc".to_string(), serde_json::to_value("hello").unwrap());
    let got = resolve_value(
        params.clone(),
        Some("abc".to_string()),
        Some("123".to_string()),
    );
    assert_eq!("hello".to_string(), got.unwrap());
    let got = resolve_value(params.clone(), Some("abc".to_string()), None);
    assert_eq!("hello".to_string(), got.unwrap());
    let got = resolve_value(
        params.clone(),
        Some("xxx".to_string()),
        Some("123".to_string()),
    );
    assert_eq!("123".to_string(), got.unwrap());
}

#[test]
fn test_extract_params() {
    let mut params = vec![];
    params.insert(
        0,
        ParameterValue {
            name: "a".to_string(),
            value: Some("1".into()),
            from_param: None,
        },
    );
    params.insert(
        1,
        ParameterValue {
            name: "b".to_string(),
            value: Some("2".into()),
            from_param: None,
        },
    );
    params.insert(
        2,
        ParameterValue {
            name: "c".to_string(),
            value: Some("3".into()),
            from_param: None,
        },
    );
    assert_eq!(
        Some("1".to_string()),
        extract_string_params("a", params.clone())
    );
    assert_eq!(
        Some("2".to_string()),
        extract_string_params("b", params.clone())
    );
    assert_eq!(
        Some("3".to_string()),
        extract_string_params("c", params.clone())
    );
    assert_eq!(None, extract_string_params("d", params.clone()));
}
