use crate::schematic::parameter::ParameterValue;
use kube::client::APIClient;

// Re-exports
mod autoscaler;
pub use crate::schematic::traits::autoscaler::Autoscaler;
mod ingress;
pub use crate::schematic::traits::ingress::Ingress;
mod util;
use crate::schematic::traits::util::*;

#[cfg(test)]
mod util_test;
#[cfg(test)]
mod autoscaler_test;
#[cfg(test)]
mod ingress_test;

/// Trait describes Hydra traits.
///
/// Hydra traits are ops-oriented "add-ons" that can be attached to Components of the appropriate workloadType.
/// For example, an autoscaler trait can attach to a workloadType (such as ReplicableService) that can be
/// scaled up and down.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trait {}

/// A TraitBinding attaches a trait to a component.
///
/// Trait bindings appear in configuration stanzas for traits.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TraitBinding {
    pub name: String,
    pub parameter_values: Option<Vec<ParameterValue>>,
}

/// HydraTrait is an enumeration of the known traits.
///
/// This is a temporary solution. In the future, we really want to be able to proxy
/// trait fulfillment down into Kubernetes and let individual trait controllers
/// fulfill the contract.
pub enum HydraTrait {
    Autoscaler(Autoscaler),
    Ingress(Ingress),
}
impl HydraTrait {
    pub fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        match self {
            HydraTrait::Autoscaler(a) => a.add(ns, client),
            HydraTrait::Ingress(i) => i.add(ns, client),
        }
    }
    pub fn delete(&self, ns: &str, client: APIClient) -> TraitResult {
        match self {
            HydraTrait::Autoscaler(a) => a.delete(ns, client),
            HydraTrait::Ingress(i) => i.delete(ns, client),
        }
    }
    pub fn modify(&self) -> TraitResult {
        match self {
            HydraTrait::Autoscaler(a) => a.modify(),
            HydraTrait::Ingress(i) => i.modify(),
        }
    }
}

/// A TraitImplementation is an implementation of a Hydra Trait.
///
/// For example, Ingress is an implementation of a Hydra Trait.
pub trait TraitImplementation {
    fn add(&self, ns: &str, client: APIClient) -> TraitResult;
    fn modify(&self) -> TraitResult {
        Err(format_err!("Trait updates not implemented for this type"))
    }
    fn delete(&self, _ns: &str, _client: APIClient) -> TraitResult {
        info!("Trait deleted");
        Ok(())
    }
    fn supports_workload_type(name: &str) -> bool {
        info!("Support {} by default", name);
        true
    }
}




