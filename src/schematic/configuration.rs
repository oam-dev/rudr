use crate::schematic::{
    traits::TraitBinding,
    parameter::ParameterValue,
};

/// Configuration creates an instance of a specified component, and attaches configuration to it.
/// 
/// In Hydra, an instance is a Component definition plus a Configuration. Practically speaking, a
/// Configuration says "Create a component of type X in scopes A, B, and C, set the following 
/// parameters, and attach these traits"
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    /// The name of the component to instantiate
    pub component: String,
    /// Values to substitute into the component
    pub parameter_values: Option<Vec<ParameterValue>>,
    /// Traits to attach to the component
    pub traits: Option<Vec<TraitBinding>>,
}