use crate::instigator::*;

#[test]
fn test_config_owner_reference() {
    let name = "configuration".to_string();
    let uid = "87707f2d-9ddc-11e9-b1c3-4ec16ac9a10f".to_string();

    config_owner_reference(name.clone(), Some(uid.clone())).and_then(|v| {
        let owner = v[0].clone();
        assert_eq!(owner.name, name);
        assert_eq!(owner.uid, uid);
        Some(v)
    }).expect("expected owner reference");
}