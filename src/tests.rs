extern crate spectral;

use crate::schematic::*;

#[test]
fn test_group_version_kind() {
    let gvk = GroupVersionKind::from_str("core.hydra.io/v1alpha1.Singleton");
    assert!(gvk.is_ok());
    let o = gvk.unwrap();
    assert_eq!("core.hydra.io", o.group);
    assert_eq!("v1alpha1", o.version);
    assert_eq!("Singleton", o.kind);

    let failed_gvp = GroupVersionKind::from_str("not valid");
    assert!(failed_gvp.is_err());
}

#[test]
fn test_component_deserialize() {
    let data = Component::from_str(
        r#"{
            "workloadType": "core.hydra.io/v1.Singleton",
            "osType": "linux",
            "arch": "amd64",
            "parameters": [],
            "containers": [],
            "workloadSettings": []
        }"#
    );

    assert!(data.is_ok());
    let component = data.unwrap();
    assert_eq!("linux", component.os_type);
    assert_eq!("amd64", component.arch);
    assert_eq!("core.hydra.io/v1.Singleton", component.workload_type);

    let gvk = GroupVersionKind::from_str(component.workload_type.as_str());
    assert!(gvk.is_ok());
}