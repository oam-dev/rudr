use kube_derive::CustomResource;
use crate::schematic::{parameter::ParameterValue, traits::TraitBinding, variable::Variable, OAMStatus};

/// Configuration creates an instance of a specified component, and attaches configuration to it.
///
/// In OAM, an instance is a Component definition plus a Configuration. Practically speaking, a
/// Configuration says "Create a component of type X in scopes A, B, and C, set the following
/// parameters, and attach these traits"
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentConfiguration {
    /// The name of the component to instantiate
    pub component_name: String,
    /// The name of the instance that is to be created
    pub instance_name: String,
    /// Values to substitute into the component
    pub parameter_values: Option<Vec<ParameterValue>>,
    /// Traits to attach to the component
    pub traits: Option<Vec<TraitBinding>>,
    /// Application Scopes which the component was involved
    pub application_scopes: Option<Vec<String>>,
}

/// ApplicationConfiguration is the top-level configuration object in OAM.
///
/// An ApplicationConfiguration can describe one or more components, a collection
/// of related parameters, and the associated traits and scopes.
#[derive(CustomResource, Serialize, Deserialize, Clone, Debug)]
#[kube(group = "core.oam.dev", version = "v1alpha1", namespaced)]
#[kube(apiextensions = "v1beta1")]
#[kube(status = "OAMStatus")]
#[serde(rename_all = "camelCase")]
pub struct ApplicationConfigurationSpec {
    pub variables: Option<Vec<Variable>>,
    pub scopes: Option<Vec<ScopeBinding>>,
    pub components: Option<Vec<ComponentConfiguration>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScopeBinding {
    pub name: String,

    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub scope_type: String,

    //TODO this should use Properties here, but we don't have Properties yet, keep consistent with TraitBinding.
    #[serde(rename(serialize = "properties", deserialize = "properties"))]
    pub parameter_values: Option<Vec<ParameterValue>>,
}
