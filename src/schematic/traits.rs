use crate::lifecycle::Phase;
use crate::schematic::parameter::ParameterValue;
use kube::client::APIClient;
use log::info;

// Re-exports
mod autoscaler;
pub use crate::schematic::traits::autoscaler::Autoscaler;
mod ingress;
pub use crate::schematic::traits::ingress::Ingress;
mod empty;
pub use crate::schematic::traits::empty::Empty;
mod manual_scaler;
pub use crate::schematic::traits::manual_scaler::ManualScaler;
mod volume_mounter;
pub use crate::schematic::traits::volume_mounter::VolumeMounter;
mod util;
use crate::schematic::traits::util::*;
use std::collections::BTreeMap;

#[cfg(test)]
mod autoscaler_test;
#[cfg(test)]
mod manual_scaler_test;
#[cfg(test)]
mod ingress_test;

pub const INGRESS_V1ALPHA1: &str = "ingress";
pub const AUTOSCALER_V1ALPHA1: &str = "auto-scaler";
pub const MANUAL_SCALER_V1ALPHA1: &str = "manual-scaler";
pub const VOLUME_MOUNTER_V1ALPHA1: &str = "volume-mounter";
pub const EMPTY: &str = "empty";

/// Trait describes OAM traits.
///
/// OAM traits are ops-oriented "add-ons" that can be attached to Components of the appropriate workloadType.
/// For example, an autoscaler trait can attach to a workloadType (such as Server) that can be
/// scaled up and down.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trait {}

/// A TraitBinding attaches a trait to a component.
///
/// Trait bindings appear in configuration stanzas for traits.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TraitBinding {
    pub name: String,
    pub parameter_values: Option<Vec<ParameterValue>>,
    pub properties: Option<serde_json::Value>,
}

/// OAMTrait is an enumeration of the known traits.
///
/// This is a temporary solution. In the future, we really want to be able to proxy
/// trait fulfillment down into Kubernetes and let individual trait controllers
/// fulfill the contract.
pub enum OAMTrait {
    Autoscaler(Autoscaler),
    ManualScaler(ManualScaler),
    Ingress(Ingress),
    VolumeMounter(Box<VolumeMounter>),
    Empty(Empty),
}
impl OAMTrait {
    pub fn exec(&self, ns: &str, client: APIClient, phase: Phase) -> TraitResult {
        match self {
            OAMTrait::Autoscaler(a) => a.exec(ns, client, phase),
            OAMTrait::Ingress(i) => i.exec(ns, client, phase),
            OAMTrait::ManualScaler(m) => m.exec(ns, client, phase),
            OAMTrait::VolumeMounter(v) => v.exec(ns, client, phase),
            OAMTrait::Empty(e) => e.exec(ns, client, phase),
        }
    }
    pub fn status(&self, ns: &str, client: APIClient) -> Option<BTreeMap<String, String>> {
        match self {
            OAMTrait::Autoscaler(a) => a.status(ns, client),
            OAMTrait::Ingress(i) => i.status(ns, client),
            OAMTrait::ManualScaler(m) => m.status(ns, client),
            OAMTrait::Empty(e) => e.status(ns, client),
            OAMTrait::VolumeMounter(v) => v.status(ns, client),
        }
    }
}

/// A TraitImplementation is an implementation of an OAM Trait.
///
/// For example, Ingress is an implementation of an OAM Trait.
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
        // Often, owner references mean you don't need to do anything here.
        // But if we invoke this delete function standalone, that means we hope to delete this sub resource actively.
        Err(format_err!("Trait delete not implemented for this type"))
    }
    fn supports_workload_type(name: &str) -> bool {
        info!("Support {} by default", name);
        true
    }
    fn pre_add(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    fn pre_modify(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    fn pre_delete(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    fn status(&self, _ns: &str, _client: APIClient) -> Option<BTreeMap<String, String>> {
        None
    }
}
