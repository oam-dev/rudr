// Re-exports
mod health;
pub use crate::schematic::scopes::health::Health;
mod network;
pub use crate::schematic::scopes::network::Network;

pub const HEALTH_SCOPE: &str = "core.hydra.io/v1alpha1.Health";
pub const NETWORK_SCOPE: &str = "core.hydra.io/v1alpha1.Network";

/// Scopes describes Hydra application scopes.
///
/// Application scopes are used to group components together into logical applications
/// by providing different forms of application boundaries with common group behaviors.
/// For example, a health scope will aggregate health states for components and determine whether it's healthy or not.
pub enum HydraScope {
    Health(Health),
    Network(Network),
}

impl HydraScope {
    pub fn allow_overlap(&self) -> bool {
        match self {
            HydraScope::Health(h) => h.allow_overlap(),
            HydraScope::Network(n) => n.allow_overlap(),
        }
    }
    pub fn scope_type(&self) -> String {
        match self {
            HydraScope::Health(h) => h.scope_type(),
            HydraScope::Network(n) => n.scope_type(),
        }
    }
}
