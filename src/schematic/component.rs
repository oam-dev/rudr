use k8s_openapi::api::apps::v1 as apps;
use k8s_openapi::api::core::v1 as core;
use k8s_openapi::apimachinery::pkg::{
    api::resource::Quantity, apis::meta::v1 as meta, util::intstr::IntOrString,
};

use std::collections::BTreeMap;

use crate::schematic::parameter::{ParameterList, ParameterType};

/// The default workload type if none is present.
pub const DEFAULT_WORKLOAD_TYPE: &str = "core.hydra.io/v1alpha1.Singleton";

/// Component describes the "spec" of a Hydra component schematic.
///
/// The wrapper of the schematic is provided by the Kubernetes library natively.
///
/// In addition to directly deserializing into a component, the from_string() helper
/// can be used for testing and prototyping.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Component {
    pub workload_type: String,
    pub os_type: String,
    pub arch: String,
    pub parameters: ParameterList,
    pub containers: Vec<Container>,
    pub workload_settings: Vec<WorkloadSetting>,
}
impl Component {
    /// Parse JSON data into a Component.
    pub fn from_str(json_data: &str) -> Result<Component, failure::Error> {
        let res: Component = serde_json::from_str(json_data)?;
        Ok(res)
    }

    /// listening_port returns the first container port listed.
    pub fn listening_port(&self) -> Option<&Port> {
        for e in self.containers.iter() {
            if let Some(first) = e.ports.iter().find_map(|p| Some(p)) {
                return Some(first);
            }
        }
        None
    }

    /// to_pod_spec generates a pod specification.
    pub fn to_pod_spec(&self) -> core::PodSpec {
        let containers = self.to_containers();
        core::PodSpec {
            containers: containers,
            ..Default::default()
        }
    }

    pub fn to_pod_spec_with_policy(&self, restart_policy: String) -> core::PodSpec {
        let containers = self.to_containers();
        core::PodSpec {
            containers: containers,
            restart_policy: Some(restart_policy),
            ..Default::default()
        }
    }

    pub fn to_containers(&self) -> Vec<core::Container> {
        self.containers
            .iter()
            .map(|c| core::Container {
                name: c.name.clone(),
                image: Some(c.image.clone()),
                resources: Some(c.resources.to_resource_requirements()),
                ports: Some(c.ports.iter().map(|p| p.to_container_port()).collect()),
                env: Some(c.env.iter().map(|e| e.to_env_var()).collect()),
                liveness_probe: c.liveness_probe.clone().and_then(|p| Some(p.to_probe())),
                readiness_probe: c.readiness_probe.clone().and_then(|p| Some(p.to_probe())),
                ..Default::default()
            })
            .collect()
    }

    pub fn to_deployment_spec(&self, name: String) -> apps::DeploymentSpec {
        let mut matching_labels = BTreeMap::new();
        matching_labels.insert("component".to_string(), name.clone());
        apps::DeploymentSpec {
            replicas: Some(1),
            selector: meta::LabelSelector {
                match_labels: Some(matching_labels.clone()),
                ..Default::default()
            },
            template: core::PodTemplateSpec {
                metadata: Some(meta::ObjectMeta {
                    labels: Some(matching_labels),
                    ..Default::default()
                }),
                spec: Some(self.to_pod_spec()),
            },
            ..Default::default()
        }
    }
}
impl Into<core::PodSpec> for Component {
    fn into(self) -> core::PodSpec {
        self.to_pod_spec()
    }
}
impl Default for Component {
    fn default() -> Self {
        Component {
            workload_type: DEFAULT_WORKLOAD_TYPE.into(),
            os_type: "linux".into(),
            arch: "amd64".into(),
            parameters: Vec::new(),
            containers: Vec::new(),
            workload_settings: Vec::new(),
        }
    }
}

/// Container describes the container configuration for a Component.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub name: String,
    pub image: String,

    #[serde(default)]
    pub resources: Resources,

    #[serde(default)]
    pub env: Vec<Env>,

    #[serde(default)]
    pub ports: Vec<Port>,

    pub liveness_probe: Option<HealthProbe>,
    pub readiness_probe: Option<HealthProbe>,
}

/// Workload settings describe the configuration for a workload.
///
/// This information is passed to the underlying workload defined by Component::worload_type.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkloadSetting {
    pub name: String,
    pub description: Option<String>,

    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub parameter_type: ParameterType,

    #[serde(default)]
    pub required: bool,

    pub default: Option<serde_json::Value>,
    pub from_param: Option<String>,
}

/// Env describes an environment variable for a container.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Env {
    pub name: String,
    pub value: Option<String>,
    pub from_param: Option<String>,
}
impl Env {
    fn to_env_var(&self) -> core::EnvVar {
        // FIXME: This needs to support fromParam
        core::EnvVar {
            name: self.name.clone(),
            value: self.value.clone(),
            value_from: None,
        }
    }
}

/// Port describes a port on a Container.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Port {
    pub name: String,
    pub container_port: i32,

    #[serde(default)]
    pub protocol: PortProtocol,
}
impl Port {
    fn to_container_port(&self) -> core::ContainerPort {
        core::ContainerPort {
            container_port: self.container_port,
            name: Some(self.name.clone()),
            protocol: Some(self.protocol.to_string()),
            ..Default::default()
        }
    }
    pub fn to_service_port(&self) -> core::ServicePort {
        core::ServicePort {
            target_port: Some(IntOrString::Int(self.container_port)),
            name: Some(self.name.clone()),
            port: 80,
            protocol: Some(self.protocol.to_string()),
            ..Default::default()
        }
    }
}

// HealthProbe describes a probe used to check on the health of a Container.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HealthProbe {
    pub exec: Option<Exec>,
    pub http_get: Option<HttpGet>,
    pub tcp_socket: Option<TcpSocket>,
    pub initial_delay_seconds: i32,
    pub period_seconds: i32,
    pub timeout_seconds: i32,
    pub success_threshold: i32,
    pub failure_threshold: i32,
}
impl HealthProbe {
    fn to_probe(&self) -> core::Probe {
        core::Probe {
            failure_threshold: Some(self.failure_threshold),
            period_seconds: Some(self.period_seconds),
            timeout_seconds: Some(self.timeout_seconds),
            success_threshold: Some(self.success_threshold),
            initial_delay_seconds: Some(self.initial_delay_seconds),
            exec: self.exec.clone().and_then(|c| {
                Some(core::ExecAction {
                    command: Some(c.command),
                })
            }),
            http_get: self
                .http_get
                .clone()
                .and_then(|a| Some(a.to_http_get_action())),
            tcp_socket: self
                .tcp_socket
                .clone()
                .and_then(|t| Some(t.to_tcp_socket_action())),
        }
    }
}
impl Default for HealthProbe {
    fn default() -> Self {
        HealthProbe {
            exec: None,
            http_get: None,
            tcp_socket: None,
            initial_delay_seconds: 0,
            period_seconds: 10,
            timeout_seconds: 1,
            success_threshold: 1,
            failure_threshold: 3,
        }
    }
}

/// Exec describes a shell command, as an array, for execution in a Container.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Exec {
    pub command: Vec<String>,
}

/// HttpGet describes an HTTP GET request used to probe a container.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HttpGet {
    pub path: String,
    pub port: i32,
    pub http_headers: Vec<HttpHeader>,
}
impl HttpGet {
    fn to_http_get_action(&self) -> core::HTTPGetAction {
        core::HTTPGetAction {
            http_headers: Some(
                self.http_headers
                    .iter()
                    .map(|h| h.to_kube_header())
                    .collect(),
            ),
            path: Some(self.path.clone()),
            port: IntOrString::Int(self.port),
            ..Default::default()
        }
    }
}

/// HttpHeader describes an HTTP header.
///
/// Headers are not stored as a map of name/value because the same header is allowed
/// multiple times.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}
impl HttpHeader {
    fn to_kube_header(&self) -> core::HTTPHeader {
        core::HTTPHeader {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

/// TcpSocket defines a socket used for health probing.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcpSocket {
    pub port: i32,
}
impl TcpSocket {
    fn to_tcp_socket_action(&self) -> core::TCPSocketAction {
        core::TCPSocketAction {
            port: IntOrString::Int(self.port),
            ..Default::default()
        }
    }
}

/// Resources defines the resources required by a container.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Resources {
    pub cpu: CPU,
    pub memory: Memory,
    pub gpu: GPU,
    pub volumes: Vec<Volume>,
}
impl Resources {
    fn to_resource_requirements(&self) -> core::ResourceRequirements {
        let mut requests = BTreeMap::new();
        requests.insert("cpu".to_string(), Quantity(self.cpu.required.clone()));
        requests.insert("memory".to_string(), Quantity(self.memory.required.clone()));

        // TODO: Kubernetes does not have a built-in type for GPUs. What do we use?
        core::ResourceRequirements {
            requests: Some(requests),
            limits: None,
        }
    }
}

impl Default for Resources {
    fn default() -> Self {
        Resources {
            cpu: CPU {
                required: "1".into(),
            },
            memory: Memory {
                required: "1G".into(),
            },
            gpu: GPU {
                required: "0".into(),
            },
            volumes: Vec::new(),
        }
    }
}

/// CPU describes a CPU resource allocation for a container.
///
/// It indicates how much CPU (core count) is required for this container to operate.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CPU {
    pub required: String,
}

/// Memory describes the memory allocation for a container.
///
/// It indicates the required amount of memory for a container to operate.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub required: String,
}

/// GPU describes a Container's need for a GPU.
///
/// It indicates how many (if any) GPU cores a container needs to operate.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GPU {
    pub required: String,
}

/// Volume describes a path that is attached to a Container.
///
/// It specifies not only the location, but also the requirements.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub name: String,
    pub mount_path: String,

    #[serde(default)]
    pub access_mode: AccessMode,

    #[serde(default)]
    pub sharing_policy: SharingPolicy,
    pub disk: Option<Disk>,
}

// Disk describes the disk requirements for backing a Volume.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Disk {
    required: String,
    ephemeral: bool,
}
impl Default for Disk {
    fn default() -> Disk {
        Disk {
            required: "1G".into(),
            ephemeral: false,
        }
    }
}

/// AccessMode defines the access modes for file systems.
///
/// Currently, only read/write and read-only are supported.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AccessMode {
    RW,
    RO,
}
impl Default for AccessMode {
    fn default() -> Self {
        AccessMode::RW
    }
}

/// SharingPolicy defines whether a filesystem can be shared across containers.
///
/// An Exclusive filesystem can only be attached to one container.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SharingPolicy {
    Shared,
    Exclusive,
}
impl Default for SharingPolicy {
    fn default() -> Self {
        SharingPolicy::Exclusive
    }
}

/// PortProtocol is a protocol used when attaching to ports.
///
/// Currently, only TCP and UDP are supported by Kubernetes.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PortProtocol {
    TCP,
    UDP,
    SCTP,
}
impl PortProtocol {
    fn as_str(&self) -> &str {
        match self {
            PortProtocol::UDP => "UDP",
            PortProtocol::SCTP => "SCTP",
            PortProtocol::TCP => "TCP",
        }
    }
}
impl Default for PortProtocol {
    fn default() -> Self {
        PortProtocol::TCP
    }
}
impl ToString for PortProtocol {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}
