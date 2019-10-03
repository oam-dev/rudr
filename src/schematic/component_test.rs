use crate::schematic::parameter::resolve_parameters;
use crate::schematic::{component::*, parameter::ParameterType, GroupVersionKind};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use std::collections::BTreeMap;
use std::str::FromStr;

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
        }"#,
    );

    assert!(data.is_ok());
    let component = data.unwrap();
    assert_eq!(Some("linux".to_string()), component.os_type);
    assert_eq!(Some("amd64".to_string()), component.arch);
    assert_eq!("core.hydra.io/v1.Singleton", component.workload_type);

    let gvk = GroupVersionKind::from_str(component.workload_type.as_str());
    assert!(gvk.is_ok());
}

#[test]
fn test_component_defaults() {
    let data = Component::from_str(r#"{"workloadType": "test"}"#);

    assert!(data.is_ok());
    let component = data.unwrap();
    assert_eq!(None, component.os_type);
    assert_eq!(None, component.arch);
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
        }"#,
    );

    assert!(data.is_ok());

    let container = data.as_ref().unwrap().containers.get(0).unwrap();
    assert_eq!("my_container", container.name);
    assert_eq!("nginx:latest", container.image);
    assert_eq!(0, container.env.len());
    assert_eq!(0, container.ports.len());
    assert!(container.cmd.is_none());
    assert!(container.args.is_none());
    assert!(container.liveness_probe.is_none());
    assert!(container.readiness_probe.is_none());
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn test_container_deserialize() {
    let def = r#"{
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
                "cmd":["nginx-debug"],
                "args":["-g","daemon off;"],
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
                    "volumes": [
                        {
                            "name": "first",
                            "mountPath": "/path/to/first"
                        },
                        {
                            "name": "second",
                            "mountPath": "/path/to/second",
                            "accessMode": "RO",
                            "sharingPolicy": "Shared"
                        }
                    ],
                    "extended": [
                        {
                            "name": "ext.example.com/v1.MotionSensor",
                            "required": "1"
                        }
                    ]
                }
            }
        ]
    }"#;
    let data = Component::from_str(def);

    assert!(data.is_ok(), "{}", data.unwrap_err());

    let container = data.as_ref().unwrap().containers.get(0).unwrap();
    assert_eq!("my_container", container.name);
    assert_eq!("nginx:latest", container.image);
    assert!(container.cmd.is_some());
    assert!(container.args.is_some());

    let cmd = container.cmd.as_ref().unwrap().get(0).unwrap();
    assert_eq!("nginx-debug", cmd);

    let args1 = container.args.as_ref().unwrap().get(0).unwrap();
    let args2 = container.args.as_ref().unwrap().get(1).unwrap();
    assert_eq!("-g", args1);
    assert_eq!("daemon off;", args2);

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

    let vols = res.volumes.clone().expect("expected volumes");
    let path1 = vols.get(0).expect("expect a first volume");
    let path2 = vols.get(1).expect("expect a second volume");

    assert_eq!("first", path1.name);
    assert_eq!("/path/to/first", path1.mount_path);
    assert_eq!(SharingPolicy::Exclusive, path1.sharing_policy);
    assert_eq!(AccessMode::RW, path1.access_mode);

    assert_eq!("second", path2.name);
    assert_eq!("/path/to/second", path2.mount_path);
    assert_eq!(SharingPolicy::Shared, path2.sharing_policy);
    assert_eq!(AccessMode::RO, path2.access_mode);

    let ext = res.extended.clone().expect("extended resources");
    let ext1 = ext.get(0).expect("expected a first resources");
    assert_eq!("ext.example.com/v1.MotionSensor", ext1.name);
    assert_eq!("1", ext1.required);
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
        }"#,
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
        }"#,
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
    assert_eq!(
        Some("things fall apart, the center cannot hold".into()),
        p1.default
    );

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
        }"#,
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
    assert_eq!(
        Some("things fall apart, the center cannot hold".into()),
        s1.default
    );

    assert_eq!("setting2", s2.name);
    assert_eq!(None, s2.description);
    assert_eq!(ParameterType::Boolean, s2.parameter_type);
    assert!(!s2.required);
    assert_eq!(None, s2.default);
    assert_eq!(Some("param1".into()), s2.from_param);
}

#[test]
fn component_listening_port() {
    let comp = Component::from_str(
        r#"{
            "containers": [
                {
                    "name": "container1",
                    "image": "nginx:latest"
                },
                {
                    "name": "container2",
                    "image": "nginx:latest",
                    "ports": [
                        {
                            "name": "good",
                            "containerPort": 443
                        },
                        {
                            "name": "bad",
                            "containerPort": 8080
                        }
                    ]
                },
                {
                    "name": "container3",
                    "image": "nginx:latest",
                    "ports": [
                        {
                            "name": "bad",
                            "containerPort": 31337
                        }
                    ]
                }
            ]
        }"#,
    );
    assert_eq!(
        "good",
        comp.expect("component should exist")
            .listening_port()
            .expect("one listening port")
            .name
    );

    assert!(Component::from_str(
        r#"{
            "containers": [
                {
                    "name": "container1",
                    "image": "nginx:latest"
                },
                {
                    "name": "container2",
                    "image": "nginx:latest"
                },
                {
                    "name": "container3",
                    "image": "nginx:latest"
                }
            ]
        }"#,
    )
    .expect("component should exist")
    .listening_port()
    .is_none());
}

#[test]
fn test_to_env_vars() {
    let env = Component::from_str(
        r#"{
            "parameters": [
                {
                    "name": "one",
                    "type": "string",
                    "default": "test one"
                },
                {
                    "name": "two",
                    "type": "number",
                    "default": 2
                },
                {
                    "name": "four",
                    "type": "number",
                    "default": 4
                }
            ],
            "containers": [
                {
                    "name": "container1",
                    "image": "nginx:latest",
                    "env": [
                        {
                            "name": "VAR_ONE",
                            "fromParam": "one"
                        },
                        {
                            "name": "VAR_TWO",
                            "fromParam": "two"
                        },
                        {
                            "name": "VAR_THREE",
                            "fromParam": "no_such_param",
                            "value": "3"
                        },
                        {
                            "name": "VAR_FOUR",
                            "fromParam": "four"
                        }
                    ]
                }
            ]
        }"#,
    )
    .as_ref()
    .expect("component should exist")
    .containers[0]
        .env
        .clone();

    let mut valmap = std::collections::BTreeMap::new();
    valmap.insert("one".to_string(), serde_json::json!("hello one"));
    valmap.insert("two".to_string(), serde_json::json!("2"));
    valmap.insert("three".to_string(), serde_json::json!("3"));

    let one = env[0].to_env_var(valmap.clone());
    let two = env[1].to_env_var(valmap.clone());
    let three = env[2].to_env_var(valmap.clone());
    let four = env[3].to_env_var(valmap.clone());

    assert_eq!("hello one", one.value.expect("found one val").as_str());
    assert_eq!("2", two.value.expect("found two val").as_str());
    assert_eq!("3", three.value.expect("found three val").as_str());

    // This is None because the valmap was never coalesced with the root values.
    assert_eq!(None, four.value);
}

#[test]
fn test_to_service_port() {
    let port = Port {
        name: "test".into(),
        container_port: 443,
        protocol: PortProtocol::TCP,
    };
    assert_eq!(443, port.to_service_port().port);
    assert_eq!(
        IntOrString::Int(443),
        port.to_service_port().target_port.expect("port")
    );
}

#[test]
fn test_to_node_seletor() {
    let data = Component::from_str(
        r#"{
            "osType":"linux",
            "arch":"amd64"
        }"#,
    );
    assert!(data.is_ok(), "Not okay: {}", data.unwrap_err());

    let comp = data.unwrap();
    let mut selector = std::collections::BTreeMap::new();
    selector.insert("kubernetes.io/os".to_string(), "linux".to_string());
    selector.insert("kubernetes.io/arch".to_string(), "amd64".to_string());
    assert_eq!(Some(selector), comp.to_node_selector())
}

#[test]
fn test_to_volume_mounts() {
    let container = Container {
        name: "test_container".into(),
        image: "test/image".into(),
        resources: Resources{
            cpu: CPU {required: "1".into()},
            memory: Memory {required: "128".into()},
            gpu: Some(GPU {required: "0".into()}),
            volumes: Some(vec![Volume{
                name: "myvol".into(),
                mount_path: "/myvol".into(),
                access_mode: AccessMode::RO,
                disk: Some(Disk{
                    ephemeral: true,
                    required: "200M".into(),
                }),
                sharing_policy: SharingPolicy::Exclusive,
            }]),
            ..Default::default()
        },
        env: vec![],
        ports: vec![],
        args: None,
        cmd: None,
        config: Some(vec![ConfigFile{
            path: "/config/file".into(),
            value: Some("value".to_string()),
            from_param: None,
        }]),
        image_pull_secret: None,
        liveness_probe: None,
        readiness_probe: None,
    };
    let mounts = container.volume_mounts();
    assert_eq!(mounts.expect("at least one mount").len(), 2);
}

#[test]
fn test_to_pod_spec_with_policy() {
    let component = Component::from_str(
        r#"{
            "osType": "linux",
            "arch": "arm64",
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
        }"#,
    )
    .expect("component must parse");

    // This is a regression test for issue #189
    {
        let map = BTreeMap::new();
        let pod = component
            .clone()
            .to_pod_spec_with_policy(map, "Always".to_string());
        let node_selector = pod.node_selector.clone().expect("node selector btree");
        assert_eq!(
            "linux".to_string(),
            *node_selector
                .get("kubernetes.io/os")
                .expect("an OS should be present")
        );
        assert_eq!(
            "arm64".to_string(),
            *node_selector
                .get("kubernetes.io/arch")
                .expect("an arch should be present")
        );
    }

    {
        let map = BTreeMap::new();
        let pod = component.to_pod_spec(map);
        let node_selector = pod.node_selector.clone().expect("node selector btree");
        assert_eq!(
            "linux".to_string(),
            *node_selector
                .get("kubernetes.io/os")
                .expect("an OS should be present")
        );
        assert_eq!(
            "arm64".to_string(),
            *node_selector
                .get("kubernetes.io/arch")
                .expect("an arch should be present")
        );
    }
}

#[test]
fn test_evaluate_configs() {
    let comp_res = Component::from_str(
        r#"{
            "parameters": [
                {
                    "name": "one",
                    "type": "string",
                    "default": "test one"
                }
            ],
            "containers": [
                {
                    "name": "container1",
                    "image": "nginx:latest"
                },
                {
                    "name": "container2",
                    "image": "nginx:latest",
                    "config": [
                        {
                            "path": "/etc/access/default_user.txt",
                            "value": "admin"
                        },
                        {
                            "path": "/var/run/db-data",
                            "fromParam": "one"
                        }
                    ]
                },
                {
                    "name": "container3",
                    "image": "nginx:latest",
                    "config": [
                        {
                            "path": "/etc/access/default_user.txt",
                            "value": "admin",
                            "fromParam": "two"
                        }
                    ]
                }
            ]
        }"#,
    );
    let comp = comp_res.as_ref().expect("component should exist");
    let resloved_val =
        resolve_parameters(comp.parameters.clone(), BTreeMap::new()).expect("resolved parameter");
    let configs = comp.evaluate_configs(resloved_val);
    assert_eq!(3, configs.len());
    let mut exp: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut c20 = BTreeMap::new();
    c20.insert("default_user.txt".to_string(), "admin".to_string());
    let mut c21 = BTreeMap::new();
    c21.insert("db-data".to_string(), "test one".to_string());
    exp.insert("container20".to_string(), c20);
    exp.insert("container21".to_string(), c21);
    let mut c30 = BTreeMap::new();
    c30.insert("default_user.txt".to_string(), "admin".to_string());
    exp.insert("container30".to_string(), c30);
    assert_eq!(exp, configs);
}

#[test]
fn test_to_deployment_spec() {
    let comp_res = Component::from_str(
        r#"{
            "parameters": [
                {
                    "name": "one",
                    "type": "string",
                    "default": "test one"
                }
            ],
            "containers": [
                {
                    "name": "container1",
                    "image": "nginx:latest"
                }
            ]
        }"#,
    );
    let comp = comp_res.as_ref().expect("component should exist");
    let mut labels = BTreeMap::new();
    labels.insert("app".to_string(), "test_deploy".to_string());
    let resloved_val =
        resolve_parameters(comp.parameters.clone(), BTreeMap::new()).expect("resolved parameter");
    let deploy = comp.to_deployment_spec(resloved_val, Some(labels.clone()), None);
    assert_eq!(deploy.selector.match_labels, Some(labels.clone()));
    assert_eq!(
        deploy.template.metadata.unwrap().labels,
        Some(labels.clone())
    );
}
