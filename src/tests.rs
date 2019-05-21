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

#[test]
fn test_component_defaults() {
    let data = Component::from_str(r#"{"workloadType": "test"}"#);

    assert!(data.is_ok());
    let component = data.unwrap();
    assert_eq!("linux", component.os_type);
    assert_eq!("amd64", component.arch);
    assert_eq!("test", component.workload_type);
    assert_eq!(0, component.parameters.len());
    assert_eq!(0, component.workload_settings.len());
    assert_eq!(0, component.containers.len());
}

#[test]
fn test_container_deserialize_defaults() {
    let data = Component::from_str(
        r#"{
            "containers": [
                {
                    "name": "my_container",
                    "image": "nginx:latest"
                }
            ]
        }"#
    );

    assert!(data.is_ok());

    let container = data.as_ref().unwrap().containers.get(0).unwrap();
    assert_eq!("my_container", container.name);
    assert_eq!("nginx:latest", container.image);
    assert_eq!(0, container.env.len());
    assert_eq!(0, container.ports.len());
    assert!(container.liveness_probe.is_none());
    assert!(container.readiness_probe.is_none());
}

#[test]
fn test_container_deserialize() {
    let data = Component::from_str(
        r#"{
            "containers": [
                {
                    "name": "my_container",
                    "image": "nginx:latest",
                    "ports": [
                        {
                            "name": "http",
                            "containerPort": 8080,
                            "protocol": "TCP"
                        },
                        {
                            "name": "admin",
                            "containerPort": 31337,
                            "protocol": "UDP"
                        }
                    ],
                    "env": [
                        {
                            "name": "key1",
                            "value": "value1"
                        },
                        {
                            "name": "key2",
                            "fromParam": "param2"
                        }
                    ],
                    "livenessProbe": {

                    }
                }
            ]
        }"#
    );

    assert!(data.is_ok());

    let container = data.as_ref().unwrap().containers.get(0).unwrap();
    assert_eq!("my_container", container.name);
    assert_eq!("nginx:latest", container.image);

    // Ports
    assert_eq!(2, container.ports.len());
    let http_port = container.ports.get(0).unwrap();
    assert_eq!("http", http_port.name);
    assert_eq!(8080, http_port.container_port);
    assert_eq!(PortProtocol::TCP, http_port.protocol);
    
    assert_eq!(2, container.env.len());
    let env1 = container.env.get(0).unwrap();
    assert_eq!("key1", env1.name);
    assert_eq!(Some("value1".to_string()), env1.value);
    assert!(env1.from_param.is_none());

    assert!(container.liveness_probe.is_some());
    assert!(container.readiness_probe.is_none());
}
#[test]
fn test_health_probe_deserialize() {
    let data = Component::from_str(
        r#"{
            "containers": [
                {
                    "name": "my_container",
                    "image": "nginx:latest",
                    "livenessProbe": {
                        "httpGet": {
                            "path": "/healthz",
                            "port": 9000,
                            "httpHeaders": [
                                {
                                    "name": "HOSTNAME",
                                    "value": "example.com"
                                }
                            ]
                        }
                    }
                }
            ]
        }"#
    );
    
    assert!(data.is_ok());

    let container = data.as_ref().unwrap().containers.get(0);
    let probe = container.unwrap().clone().liveness_probe.unwrap();
    
    assert_eq!(10, probe.period_seconds);
    assert_eq!(1, probe.http_get.as_ref().unwrap().http_headers.len());
    assert_eq!(9000, probe.http_get.as_ref().unwrap().port);
}