/// Parameter describes a configurable unit on a Component or Application.
/// 
/// Parameters have primitive types, and may be marked as required. Default values
/// may be provided as well.
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

/// Supplies the default value for all required fields.
fn default_required() -> bool {
    false
}

/// ParameterType defines the types of parameters for a Parameters object.
///
/// These roughly correlate with JSON Schema primitive types.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ParameterType {
    Boolean,
    String,
    Number,
    Null,
}

/// A value that is substituted into a parameter.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParameterValue {
    name: String,
    value: serde_json::Value,
}