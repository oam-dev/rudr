
/// Component describes the "spec" of a Hydra component schematic.
/// 
/// The wrapper of the schematic is provided by the Kubernetes library natively.
/// 
/// In addition to directly deserializing into a component, the from_string() helper
/// can be used for testing and prototyping.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub workload_type: String,
    pub os_type: String,
    pub arch: String,
    pub parameters: Vec<Parameter>,
    pub containers: Vec<Container>,
    pub workload_settings: Vec<WorkloadSetting>,
}

impl Component {
    pub fn from_str(json_data: &str) -> Result<Component, failure::Error> {
        let res: Component = serde_json::from_str(json_data)?;
        Ok(res)
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
    name: String,
    description: Option<String>,
    parameter_type: ParameterType,
    required: bool,
    default: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    name: String,
    image: String,
    resources: Resources,
    env: Vec<Env>,
    ports: Vec<Port>,
    liveness_probe: HealthProbe,
    readiness_probe: HealthProbe,
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
    name: String,
    value: Option<String>,
    from_param: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Port{
    name: String,
    container_port: i32,
    protocol: PortProtocol,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HealthProbe{
    exec: Option<Exec>,
    http_get: Option<HttpGet>,
    tcp_socket: Option<TcpSocket>,
    initial_delay_seconds: i64,
    period_seconds: i64,
    timeout_seconds: i64,
    success_threshold: i64,
    failure_threshold: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Exec {
    command: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HttpGet{
    path: String,
    port: i64,
    http_headers: Vec<HttpHeader>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader{
    name: String,
    value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcpSocket{
    port: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resources{
    cpu: CPU,
    memory: Memory,
    gpu: GPU,
    path: Vec<Path>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CPU{
    required: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Memory{
    required: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GPU{
    required: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Path{
    name: String,
    path: String,
    access_mode: AccessMode,
    sharing_policy: SharingPolicy,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ParameterType {
    Boolean,
    String,
    Number,
    Null,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum AccessMode{
    RW,
    RO,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SharingPolicy{
    Shared,
    Exclusive,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PortProtocol{
    TCP,
    UDP,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HydraStatus {
    phase: Option<String>,
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
    pub fn new(group: &str, version: &str, kind: &str) -> GroupVersionKind {
        GroupVersionKind{
            group: group.into(),
            version: version.into(),
            kind: kind.into(),
        }
    }
    pub fn from_str(gvp: &str) -> Result<GroupVersionKind, failure::Error> {
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