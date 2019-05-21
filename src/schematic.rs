
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
    pub parameters: Vec<Parameter>,
    pub containers: Vec<Container>,
    pub workload_settings: Vec<WorkloadSetting>,
}

impl Component {
    /// Parse JSON data into a Component.
    pub fn from_str(json_data: &str) -> Result<Component, failure::Error> {
        let res: Component = serde_json::from_str(json_data)?;
        Ok(res)
    }
}

/// The default workload type if none is present.
pub const DEFAULT_WORKLOAD_TYPE: &str = "core.hydra.io/v1alpha1.Singleton";

impl Default for Component {
    fn default() -> Self {
        Component{
            workload_type: DEFAULT_WORKLOAD_TYPE.into(),
            os_type: "linux".into(),
            arch: "amd64".into(),
            parameters: Vec::new(),
            containers: Vec::new(),
            workload_settings: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Application {

}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trait {

}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub description: Option<String>,

    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub parameter_type: ParameterType,

    #[serde(default = "default_required")]
    pub required: bool,

    pub default: Option<serde_json::Value>,
}

fn default_required() -> bool {
    false
}

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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkloadSetting{
    name: String,
    parameter_type: ParameterType,
    value: Option<serde_json::Value>,
    from_param: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Env{
    pub name: String,
    pub value: Option<String>,
    pub from_param: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Port{
    pub name: String,
    pub container_port: i32,

    #[serde(default)]
    pub protocol: PortProtocol,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct HealthProbe{
    pub exec: Option<Exec>,
    pub http_get: Option<HttpGet>,
    pub tcp_socket: Option<TcpSocket>,
    pub initial_delay_seconds: i64,
    pub period_seconds: i64,
    pub timeout_seconds: i64,
    pub success_threshold: i64,
    pub failure_threshold: i64,
}

impl Default for HealthProbe {
    fn default() -> Self {
        HealthProbe{
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Exec {
    pub command: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HttpGet{
    pub path: String,
    pub port: i64,
    pub http_headers: Vec<HttpHeader>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader{
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcpSocket{
    pub port: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resources{
    pub cpu: CPU,
    pub memory: Memory,
    pub gpu: GPU,
    pub path: Vec<Path>,
}

impl Default for Resources {
    fn default() -> Self {
        Resources {
            cpu: CPU{required: "1".into()},
            memory: Memory{required: "1G".into()},
            gpu: GPU{required: "0".into()},
            path: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CPU{
    pub required: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Memory{
    pub required: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GPU{
    pub required: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Path{
    pub name: String,
    pub path: String,
    pub access_mode: AccessMode,
    pub sharing_policy: SharingPolicy,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ParameterType {
    Boolean,
    String,
    Number,
    Null,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum AccessMode{
    RW,
    RO,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SharingPolicy{
    Shared,
    Exclusive,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PortProtocol{
    TCP,
    UDP,
}
impl Default for PortProtocol {
    fn default() -> Self {
        PortProtocol::TCP
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HydraStatus {
    phase: Option<String>,
}

impl Default for HydraStatus {
    fn default() -> Self {
        HydraStatus {
            phase: None,
        }
    }
}

pub type Status = Option<HydraStatus>;

pub struct GroupVersionKind {
    pub group: String,
    pub version: String,
    pub kind: String,
}

/// GroupVersionKind represents a canonical name, composed of group, version, and (you guessed it) kind.
/// 
/// Group is a dotted name. While the specification requires at least one dot in the group, we do not enforce.
/// Version is an API version
/// Kind the name of the type
impl GroupVersionKind {
    /// Create a new GroupVersionKind from each component.
    /// 
    /// This does not check the formatting of each part.
    pub fn new(group: &str, version: &str, kind: &str) -> GroupVersionKind {
        GroupVersionKind{
            group: group.into(),
            version: version.into(),
            kind: kind.into(),
        }
    }
    /// Parse a string into a GroupVersionKind.
    pub fn from_str(gvp: &str) -> Result<GroupVersionKind, failure::Error> {
        // I suspect that this function could be made much more elegant.
        let parts: Vec<&str> = gvp.splitn(2, "/").collect();
        if parts.len() != 2 {
            return Err(failure::err_msg("missing version and kind"))
        }

        let vk: Vec<&str> = parts.get(1).unwrap().splitn(2, ".").collect();
        if vk.len() != 2 {
            return Err(failure::err_msg("missing kind"))
        }

        Ok(GroupVersionKind{
            group: parts.get(0).unwrap().to_string(),
            version: vk.get(0).unwrap().to_string(),
            kind: vk.get(1).unwrap().to_string(),
        })
    }
}