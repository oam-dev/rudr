use crate::schematic::{
    traits::TraitBinding,
    parameter::ParameterValue,
};

/// Configuration creates an instance of a specified component, and attaches configuration to it.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    /// The name of the component to instantiate
    pub component: String,
    /// Values to substitute into the component
    pub parameter_values: Vec<ParameterValue>,
    /// Traits to attach to the component
    pub traits: Vec<TraitBinding>,
}