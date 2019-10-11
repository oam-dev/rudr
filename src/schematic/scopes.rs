// Re-exports
mod health;
pub use crate::schematic::scopes::health::Health;
mod network;
use crate::schematic::configuration::ComponentConfiguration;
pub use crate::schematic::scopes::network::Network;
use failure::Error;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;

pub const HEALTH_SCOPE: &str = "core.hydra.io/v1alpha1.HealthScope";
pub const NETWORK_SCOPE: &str = "core.hydra.io/v1alpha1.NetworkScope";

/// Scopes describes Hydra application scopes.
///
/// Application scopes are used to group components together into logical applications
/// by providing different forms of application boundaries with common group behaviors.
/// For example, a health scope will aggregate health states for components and determine whether it's healthy or not.
pub enum OAMScope {
    Health(Health),
    Network(Network),
}

fn convert_owner_ref(owner: meta::OwnerReference) -> kube::api::OwnerReference {
    kube::api::OwnerReference {
        controller: owner.controller.unwrap_or(false),
        blockOwnerDeletion: owner.block_owner_deletion.unwrap_or(false),
        name: owner.name,
        apiVersion: owner.api_version,
        kind: owner.kind,
        uid: owner.uid,
    }
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
    pub fn create(&self, owner: meta::OwnerReference) -> Result<(), Error> {
        match self {
            OAMScope::Health(h) => h.create(convert_owner_ref(owner.clone())),
            OAMScope::Network(n) => n.create(owner.clone()),
        }
    }
    /// modify will modify the scope instance
    pub fn modify(&self) -> Result<(), Error> {
        match self {
            OAMScope::Health(h) => h.modify(),
            OAMScope::Network(n) => n.modify(),
        }
    }
    /// delete will delete the scope instance, we can depend on OwnerReference if only k8s objects were created
    pub fn delete(&self) -> Result<(), Error> {
        match self {
            OAMScope::Health(h) => h.delete(),
            OAMScope::Network(n) => n.delete(),
        }
    }
    /// add will add a component to this scope
    pub fn add(&self, spec: ComponentConfiguration) -> Result<(), Error> {
        match self {
            OAMScope::Health(h) => h.add(spec),
            OAMScope::Network(n) => n.add(spec),
        }
    }
    /// remove will remove component from this scope
    pub fn remove(&self, spec: ComponentConfiguration) -> Result<(), Error> {
        match self {
            OAMScope::Health(h) => h.remove(spec),
            OAMScope::Network(n) => n.remove(spec),
        }
    }
}
