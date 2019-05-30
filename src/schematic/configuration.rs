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
pub struct ComponentConfiguration {
    /// The name of the component to instantiate
    pub name: String,
    /// Values to substitute into the component
    pub parameter_values: Option<Vec<ParameterValue>>,
    /// Traits to attach to the component
    pub traits: Option<Vec<TraitBinding>>,
}

/// OperationalConfiguration is the top-level configuration object in Hydra.
/// 
/// An OperationalConfiguration can describe one or more components, a collection
/// of related parameters, and the associated traits and scopes.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OperationalConfiguration {
    pub parameter_values: Option<Vec<ParameterValue>>,
    pub scopes: Option<Vec<ScopeBinding>>,
    pub components: Option<Vec<ComponentConfiguration>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScopeBinding{
    pub name: String,

    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub scope_type: String,

    pub parameter_values: Option<Vec<ParameterValue>>,
}