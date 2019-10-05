use crate::schematic::configuration::*;

#[test]
fn test_component_configuration() {
    // Test that a configuration deserializes correctly.

    let conf: ComponentConfiguration = serde_json::from_str(
        r#"{
            "name": "test",
            "instanceName": "squidgy",
            "parameterValues": [
                {
                    "name": "param1",
                    "value": 1234
                }
            ]
        }"#,
    )
    .expect("JSON must parse");

    assert_eq!("test", conf.name);
    assert_eq!("squidgy", conf.instance_name);
    assert!(conf.parameter_values.is_some());
    assert!(conf.traits.is_none());
}

#[test]
fn test_application_configuration() {
    // Test that an application configuration deserializes correctly.

    let conf: ApplicationConfiguration = serde_json::from_str(
        r#"{
            "variables": [
                {
                    "name": "var1",
                    "value": 1234
                }
            ]
        }"#,
    )
    .expect("JSON must parse");

    assert!(conf.variables.is_some());
}
