use crate::instigator::*;

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
