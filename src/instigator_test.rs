use crate::instigator::*;
use crate::schematic::configuration::ComponentConfiguration;
use std::collections::BTreeMap;

#[test]
fn test_config_owner_reference() {
    let name = "configuration".to_string();
    let uid = "87707f2d-9ddc-11e9-b1c3-4ec16ac9a10f".to_string();

    config_owner_reference(name.clone(), Some(uid.clone()))
        .and_then(|owner| {
            assert_eq!(owner.name, name);
            assert_eq!(owner.uid, uid);
            Ok(owner)
        })
        .expect("expected owner reference");
}

#[test]
fn test_record_ann() {
    let records = None;
    let ann = get_record_annotation(records).expect("get_record_annotation from none");
    assert_eq!(ann.len(), 0);
    let mut one: RecordAnnotation = BTreeMap::new();
    let cr = ComponentRecord {
        version: "123".to_string(),
        config: ComponentConfiguration {
            name: "n123".to_string(),
            instance_name: "inst123".to_string(),
            parameter_values: None,
            traits: None,
        },
    };
    let cr2 = ComponentRecord {
        version: "321".to_string(),
        config: ComponentConfiguration {
            name: "n321".to_string(),
            instance_name: "inst321".to_string(),
            parameter_values: None,
            traits: None,
        },
    };
    one.insert("comp1".to_string(), cr.clone());
    one.insert("comp2".to_string(), cr2.clone());
    let json_str = &serde_json::to_string(&one).expect("record annotation json value");
    let records = Some(json_str);
    let ann = get_record_annotation(records).expect("get_record_annotation from json");
    assert_eq!(2, ann.len());
    let crgot = ann.get("comp1").expect("comp1 ComponentRecord");
    let crgot2 = ann.get("comp2").expect("comp2 ComponentRecord");
    assert_eq!(crgot, &cr);
    assert_eq!(crgot2, &cr2);
}

#[test]
fn test_check_diff() {
    let new_record = ComponentRecord {
        version: "123".to_string(),
        config: ComponentConfiguration {
            name: "test".to_string(),
            instance_name: "test_inst".to_string(),
            parameter_values: None,
            traits: None,
        },
    };
    let old_record = ComponentRecord {
        version: "123".to_string(),
        config: ComponentConfiguration {
            name: "test".to_string(),
            instance_name: "test_inst".to_string(),
            parameter_values: None,
            traits: None,
        },
    };

    assert_eq!(check_diff(None, &new_record), true);
    assert_eq!(check_diff(Some(old_record.clone()), &new_record), false);

    let new_record2 = ComponentRecord {
        version: "1234".to_string(),
        config: ComponentConfiguration {
            name: "test".to_string(),
            instance_name: "test_inst".to_string(),
            parameter_values: None,
            traits: None,
        },
    };
    assert_eq!(check_diff(Some(new_record2), &old_record), true);
    let new_record3 = ComponentRecord {
        version: "1234".to_string(),
        config: ComponentConfiguration {
            name: "test".to_string(),
            instance_name: "test_inst".to_string(),
            parameter_values: Some(vec![]),
            traits: None,
        },
    };
    assert_eq!(check_diff(Some(new_record3), &old_record), true);
}
