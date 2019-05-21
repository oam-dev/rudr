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
                    "livenessProbe": {},
                    "resources": {
                        "memory": {
                            "required": "2G"
                        },
                        "paths": [
                            {
                                "name": "first",
                                "path": "/path/to/first"
                            },
                            {
                                "name": "second",
                                "path": "/path/to/second",
                                "accessMode": "RO",
                                "sharingPolicy": "Shared"
                            }
                        ]
                    }
                }
            ]
        }"#
    );

    assert!(data.is_ok(), "{}", data.unwrap_err());

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

    let res = &container.resources;

    assert_eq!("2G", res.memory.required);
    assert_eq!("1", res.cpu.required);

    let path1 = res.paths.get(0).unwrap();
    let path2 = res.paths.get(1).unwrap();

    assert_eq!("first", path1.name);
    assert_eq!("/path/to/first", path1.path);
    assert_eq!(SharingPolicy::Exclusive, path1.sharing_policy);
    assert_eq!(AccessMode::RW, path1.access_mode);

    assert_eq!("second", path2.name);
    assert_eq!("/path/to/second", path2.path);
    assert_eq!(SharingPolicy::Shared, path2.sharing_policy);
    assert_eq!(AccessMode::RO, path2.access_mode);
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

    let headers = &probe.http_get.as_ref().unwrap().http_headers;
    assert_eq!(1, headers.len());
    assert_eq!("HOSTNAME", headers.get(0).unwrap().name);
    assert_eq!("example.com", headers.get(0).unwrap().value);
    assert_eq!(9000, probe.http_get.as_ref().unwrap().port);
}

#[test]
fn test_parameter_deserialize() {
   let data = Component::from_str(
        r#"{
            "parameters": [
                {
                    "name": "param1",
                    "description": "a parameter",
                    "type": "string",
                    "required": true,
                    "default": "things fall apart, the center cannot hold"
                },
                {
                    "name": "param2",
                    "type": "boolean"
                }
            ]
        }"#
    );

    assert!(data.is_ok(), "Not okay: {}", data.unwrap_err());

    let params = data.unwrap().parameters;

    assert_eq!(2, params.len());

    let p1 = params.get(0).unwrap();
    let p2 = params.get(1).unwrap();

    assert_eq!("param1", p1.name);
    assert_eq!(Some("a parameter".into()), p1.description);
    assert_eq!(ParameterType::String, p1.parameter_type);
    assert!(p1.required);
    assert_eq!(Some("things fall apart, the center cannot hold".into()), p1.default);

    assert_eq!("param2", p2.name);
    assert_eq!(None, p2.description);
    assert_eq!(ParameterType::Boolean, p2.parameter_type);
    assert!(!p2.required);
    assert_eq!(None, p2.default);
}

#[test]
fn test_workload_settings_deserialize() {
   let data = Component::from_str(
        r#"{
            "workloadSettings": [
                {
                    "name": "setting1",
                    "description": "a workload setting",
                    "type": "string",
                    "required": true,
                    "default": "things fall apart, the center cannot hold"
                },
                {
                    "name": "setting2",
                    "type": "boolean",
                    "fromParam": "param1"
                }
            ]
        }"#
    );

    assert!(data.is_ok(), "Not okay: {}", data.unwrap_err());

    let settings = data.unwrap().workload_settings;

    assert_eq!(2, settings.len());

    let s1 = settings.get(0).unwrap();
    let s2 = settings.get(1).unwrap();

    assert_eq!("setting1", s1.name);
    assert_eq!(Some("a workload setting".into()), s1.description);
    assert_eq!(ParameterType::String, s1.parameter_type);
    assert!(s1.required);
    assert_eq!(Some("things fall apart, the center cannot hold".into()), s1.default);

    assert_eq!("setting2", s2.name);
    assert_eq!(None, s2.description);
    assert_eq!(ParameterType::Boolean, s2.parameter_type);
    assert!(!s2.required);
    assert_eq!(None, s2.default);
    assert_eq!(Some("param1".into()), s2.from_param);
}