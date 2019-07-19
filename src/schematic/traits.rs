use crate::schematic::parameter::ParameterValue;
use crate::lifecycle::Phase;
use kube::client::APIClient;

// Re-exports
mod autoscaler;
pub use crate::schematic::traits::autoscaler::Autoscaler;
mod ingress;
pub use crate::schematic::traits::ingress::Ingress;
mod empty;
pub use crate::schematic::traits::empty::Empty;
mod manual_scaler;
pub use crate::schematic::traits::manual_scaler::ManualScaler;
mod util;
use crate::schematic::traits::util::*;

#[cfg(test)]
mod util_test;
#[cfg(test)]
mod autoscaler_test;
#[cfg(test)]
mod ingress_test;
#[cfg(test)]
mod manual_scaler_test;

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
    ManualScaler(ManualScaler),
    Ingress(Ingress),
    Empty(Empty),
}
impl HydraTrait {
    pub fn exec(&self, ns: &str, client: APIClient, phase: Phase) -> TraitResult {
        match self {
            HydraTrait::Autoscaler(a) => a.exec(ns, client, phase),
            HydraTrait::Ingress(i) => i.exec(ns, client, phase),
            HydraTrait::ManualScaler(m) => m.exec(ns, client, phase),
            HydraTrait::Empty(e) => e.exec(ns, client, phase),
        }
    }
}

/// A TraitImplementation is an implementation of a Hydra Trait.
///
/// For example, Ingress is an implementation of a Hydra Trait.
pub trait TraitImplementation {
    fn exec(&self, ns: &str, client: APIClient, phase: Phase) -> TraitResult {
        match phase {
            Phase::Add => self.add(ns, client),
            Phase::Modify => self.modify(ns, client),
            Phase::Delete => self.delete(ns, client),
            Phase::PreAdd => self.pre_add(ns, client),
            Phase::PreModify => self.pre_modify(ns, client),
            Phase::PreDelete => self.pre_delete(ns, client),
        }
    }
    fn add(&self, ns: &str, client: APIClient) -> TraitResult;
    fn modify(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Err(format_err!("Trait updates not implemented for this type"))
    }
    fn delete(&self, _ns: &str, _client: APIClient) -> TraitResult {
        // Often, owner references mean you don't need to do anythign here.
        info!("Trait deleted");
        Ok(())
    }
    fn supports_workload_type(name: &str) -> bool {
        info!("Support {} by default", name);
        true
    }
    fn pre_add(&self, _ns: &str, _client: APIClient)  -> TraitResult {
        Ok(())
    }
    fn pre_modify(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    fn pre_delete(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
}




