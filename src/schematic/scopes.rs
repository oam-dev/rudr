use crate::schematic::parameter::ParameterValue;

// Re-exports
mod health;
pub use crate::schematic::scopes::health::Health;
mod network;
pub use crate::schematic::scopes::network::Network;

/// Scopes describes Hydra application scopes.
///
/// Application scopes are used to group components together into logical applications
/// by providing different forms of application boundaries with common group behaviors.
/// For example, a health scope will aggregate health states for components and determine whether it's healthy or not.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub scope_type: String,
    pub allow_component_overlap: bool,
    pub parameter_values: Option<Vec<ParameterValue>>,
}

pub enum HydraScope {
    Health(Health),
    // Network(Network),
}

impl HydraScope {
    pub fn allow_overlap(&self) -> bool {
        match self {
            HydraScope::Health(h) => h.allow_overlap(),
            // HydraScope::Network(n) => n.allow_overlap(),
        }
    }
    pub fn scope_type(&self) -> String {
        match self {
            HydraScope::Health(h) => h.scope_type(),
            //     HydraScope::Network(n) => n.allow_overlap(),
        }
    }
}
