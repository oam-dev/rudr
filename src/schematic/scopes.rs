// Re-exports
mod health;
pub use crate::schematic::scopes::health::Health;
mod network;
pub use crate::schematic::scopes::network::Network;
use failure::Error;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;

pub const HEALTH_SCOPE: &str = "core.hydra.io/v1alpha1.Health";
pub const NETWORK_SCOPE: &str = "core.hydra.io/v1alpha1.Network";

/// Scopes describes Hydra application scopes.
///
/// Application scopes are used to group components together into logical applications
/// by providing different forms of application boundaries with common group behaviors.
/// For example, a health scope will aggregate health states for components and determine whether it's healthy or not.
pub enum OAMScope {
    Health(Health),
    Network(Network),
}

impl OAMScope {
    pub fn allow_overlap(&self) -> bool {
        match self {
            OAMScope::Health(h) => h.allow_overlap(),
            OAMScope::Network(n) => n.allow_overlap(),
        }
    }
    pub fn scope_type(&self) -> String {
        match self {
            OAMScope::Health(h) => h.scope_type(),
            OAMScope::Network(n) => n.scope_type(),
        }
    }
    /// create will create a real scope instance
    pub fn create(&self, ns: &str, owner: meta::OwnerReference) -> Result<(), Error> {
        match self {
            OAMScope::Health(h) => h.create(ns, owner.clone()),
            OAMScope::Network(n) => n.create(ns, owner.clone()),
        }
    }
    /// modify will modify the scope instance
    pub fn modify(&self, ns: &str) -> Result<(), Error> {
        match self {
            OAMScope::Health(h) => h.modify(ns),
            OAMScope::Network(n) => n.modify(ns),
        }
    }
    /// delete will delete the scope instance
    pub fn delete(&self, ns: &str) -> Result<(), Error> {
        match self {
            OAMScope::Health(h) => h.delete(ns),
            OAMScope::Network(n) => n.delete(ns),
        }
    }
}
