
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    workload_type: String,
    os_type: String,
    arch: String,
    parameters: Vec<Parameter>,
    containers: Vec<Container>,
    workload_settings: Vec<WorkloadSetting>,
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