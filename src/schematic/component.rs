use k8s_openapi::api::apps::v1 as apps;
use k8s_openapi::api::core::v1 as core;
use k8s_openapi::apimachinery::pkg::{
    api::resource::Quantity, apis::meta::v1 as meta, util::intstr::IntOrString,
};
use log::info;
use std::collections::BTreeMap;
use std::path::Path;

use crate::schematic::parameter::{resolve_value, ParameterList, ParameterType, ResolvedVals};

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
    pub os_type: Option<String>,
    pub arch: Option<String>,
    pub parameters: ParameterList,
    pub containers: Vec<Container>,
    pub workload_settings: Vec<WorkloadSetting>,
}
impl Component {
    /// listening_port returns the first container port listed.
    pub fn listening_port(&self) -> Option<&Port> {
        // Iterate through each container, and if any one contains a port,
        // return it and stop processing.
        self.containers
            .iter()
            .find_map(|e| e.ports.iter().find_map(Some))
    }

    pub fn to_node_selector(&self) -> Option<BTreeMap<String, String>> {
        let mut selector = BTreeMap::new();
        if let Some(os) = self.os_type.clone() {
            selector.insert("kubernetes.io/os".to_string(), os);
        }
        if let Some(arch) = self.arch.clone() {
            selector.insert("kubernetes.io/arch".to_string(), arch);
        }
        if selector.is_empty() {
            return None;
        }
        Some(selector)
    }

    /// to_pod_spec generates a pod specification.
    pub fn to_pod_spec(&self, param_vals: ResolvedVals) -> core::PodSpec {
        let containers = self.to_containers(param_vals);
        let image_pull_secrets = Some(self.image_pull_secrets());
        let node_selector = self.to_node_selector();
        let mut vols = vec![];
        for container in self.containers.iter() {
            for (i, _conf) in container
                .config
                .clone()
                .unwrap_or_else(|| vec![])
                .iter()
                .enumerate()
            {
                vols.push(core::Volume {
                    config_map: Some(core::ConfigMapVolumeSource {
                        name: Some(container.name.clone() + i.to_string().as_str()),
                        ..Default::default()
                    }),
                    name: container.name.clone() + i.to_string().as_str(),
                    ..Default::default()
                });
            }
            container
                .resources
                .volumes
                .clone()
                .unwrap_or_else(|| vec![])
                .iter()
                .for_each(|v| {
                    // There is an ephemeral flag on v.disk. What do we do with that?
                    vols.push(core::Volume {
                        name: v.name.clone(),
                        empty_dir: Some(core::EmptyDirVolumeSource {
                            size_limit: v.disk.clone().and_then(|d| Some(Quantity(d.required))),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                })
        }
        let volumes = Some(vols);
        core::PodSpec {
            containers,
            image_pull_secrets,
            node_selector,
            volumes,
            ..Default::default()
        }
    }

    pub fn to_pod_spec_with_policy(
        &self,
        param_vals: ResolvedVals,
        restart_policy: String,
    ) -> core::PodSpec {
        let mut pod_spec = self.to_pod_spec(param_vals);
        pod_spec.restart_policy = Some(restart_policy);
        pod_spec
    }

    pub fn evaluate_configs(
        &self,
        resolved_vals: ResolvedVals,
    ) -> BTreeMap<String, BTreeMap<String, String>> {
        let mut configs: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        for container in self.containers.iter() {
            for (i, conf) in container
                .config
                .clone()
                .unwrap_or_else(|| vec![])
                .iter()
                .enumerate()
            {
                let mut values = BTreeMap::new();
                //config name is file path
                let filename = Path::new(conf.path.as_str())
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .expect("config file name");
                values.insert(filename, conf.resolve_value(resolved_vals.clone()));
                configs.insert(container.name.clone() + i.to_string().as_str(), values);
            }
        }
        configs
    }

    pub fn to_containers(&self, resolved_vals: ResolvedVals) -> Vec<core::Container> {
        self.containers
            .iter()
            .map(|c| core::Container {
                name: c.name.clone(),
                image: Some(c.image.clone()),
                resources: Some(c.resources.to_resource_requirements()),
                ports: Some(c.ports.iter().map(|p| p.to_container_port()).collect()),
                command: c.cmd.clone(),
                args: c.args.clone(),
                env: Some(
                    c.env
                        .iter()
                        .map(|e| e.to_env_var(resolved_vals.clone()))
                        .collect(),
                ),

                volume_mounts: c.volume_mounts(),
                liveness_probe: c.liveness_probe.clone().and_then(|p| Some(p.to_probe())),
                readiness_probe: c.readiness_probe.clone().and_then(|p| Some(p.to_probe())),
                ..Default::default()
            })
            .collect()
    }

    pub fn image_pull_secrets(&self) -> Vec<core::LocalObjectReference> {
        self.containers
            .iter()
            .filter_map(|c| {
                info!("Looking for image pull secret");
                c.image_pull_secret.clone().and_then(|n| {
                    info!("found image pull secret");
                    Some(core::LocalObjectReference { name: Some(n) })
                })
            })
            .collect()
    }

    pub fn to_deployment_spec(
        &self,
        param_vals: ResolvedVals,
        labels: Option<BTreeMap<String, String>>,
        annotations: Option<BTreeMap<String, String>>,
    ) -> apps::DeploymentSpec {
        apps::DeploymentSpec {
            replicas: Some(1),
            selector: meta::LabelSelector {
                match_labels: labels.clone(),
                ..Default::default()
            },
            template: core::PodTemplateSpec {
                metadata: Some(meta::ObjectMeta {
                    annotations,
                    labels: labels.clone(),
                    ..Default::default()
                }),
                spec: Some(self.to_pod_spec(param_vals)),
            },
            ..Default::default()
        }
    }
}

impl Default for Component {
    fn default() -> Self {
        Component {
            workload_type: DEFAULT_WORKLOAD_TYPE.into(),
            os_type: None,
            arch: None,
            parameters: Vec::new(),
            containers: Vec::new(),
            workload_settings: Vec::new(),
        }
    }
}

impl std::str::FromStr for Component {
    type Err = failure::Error;

    /// Parse JSON data into a Component.
    fn from_str(json_data: &str) -> Result<Self, Self::Err> {
        let res: Component = serde_json::from_str(json_data)?;
        Ok(res)
    }
}

/// Container describes the container configuration for a Component.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub name: String,
    pub image: String,
    pub image_pull_secret: Option<String>,

    #[serde(default)]
    pub resources: Resources,

    pub cmd: Option<Vec<String>>,
    pub args: Option<Vec<String>>,

    #[serde(default)]
    pub env: Vec<Env>,

    #[serde(default)]
    pub config: Option<Vec<ConfigFile>>,

    #[serde(default)]
    pub ports: Vec<Port>,

    pub liveness_probe: Option<HealthProbe>,
    pub readiness_probe: Option<HealthProbe>,
}

impl Container {
    /// Generate volume mounts for a container.
    pub fn volume_mounts(&self) -> Option<Vec<core::VolumeMount>> {
        let mut volumes: std::vec::Vec<core::VolumeMount> = self.config.clone().map_or(vec![], |p| {
            p.iter().enumerate().map(|(i, v)| {
                self.volume_mount(i, v)
            }).collect()
        });
        self.resources
            .volumes
            .clone()
            .unwrap_or_else(|| vec![])
            .iter()
            .for_each(|vol| {
                volumes.push(core::VolumeMount {
                    mount_path: vol.mount_path.clone(),
                    name: vol.name.clone(),
                    read_only: Some(vol.access_mode == AccessMode::RO),
                    ..Default::default()
                })
            });
        match volumes.len() {
            0 => None,
            _ => Some(volumes),
        }
    }

    fn volume_mount(&self, file_index: usize, config_file: &ConfigFile) -> core::VolumeMount {
        let path = Path::new(config_file.path.as_str())
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        core::VolumeMount {
            mount_path: path,
            name: self.name.clone() + file_index.to_string().as_str(),
            ..Default::default()
        }
    }
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

/// ConfigFile describes locations to write configuration as files accessible within the container
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFile {
    pub path: String,
    pub value: Option<String>,
    pub from_param: Option<String>,
}
impl ConfigFile {
    pub(crate) fn resolve_value(&self, params: ResolvedVals) -> String {
        let value = resolve_value(params, self.from_param.clone(), self.value.clone());
        // rely on pre-check: that one of the value or from_param must exist.
        value.unwrap()
    }
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
    pub(crate) fn to_env_var(&self, params: ResolvedVals) -> core::EnvVar {
        let value = resolve_value(params, self.from_param.clone(), self.value.clone());
        // FIXME: This needs to support fromParam
        core::EnvVar {
            name: self.name.clone(),
            value,
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
        let port = self.container_port;
        core::ServicePort {
            port,
            target_port: Some(IntOrString::Int(port)),
            name: Some(self.name.clone()),
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

type ExtendedResources = Vec<ExtendedResource>;

/// Resources defines the resources required by a container.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Resources {
    pub cpu: Option<CPU>,
    pub memory: Option<Memory>,
    pub gpu: Option<GPU>,
    pub volumes: Option<Vec<Volume>>,
    pub extended: Option<ExtendedResources>,
}

impl Resources {
    fn to_resource_requirements(&self) -> core::ResourceRequirements {
        let mut requests = BTreeMap::new();

        self.cpu.clone().and_then(|cpu|{
            requests.insert("cpu".to_string(), Quantity(cpu.required.clone()))
        });
        self.memory.clone().and_then(|mem|{
            requests.insert("memory".to_string(), Quantity(mem.required.clone()))
        });

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
            cpu: None,
            memory: None,
            gpu: None,
            volumes: None,
            extended: None,
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
    pub required: String,
    pub ephemeral: bool,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtendedResource {
    pub name: String,
    pub required: String,
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
