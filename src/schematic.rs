use failure::{err_msg, Error};

pub mod component;
pub mod component_instance;
pub mod configuration;
pub mod parameter;
pub mod scopes;
pub mod traits;

#[cfg(test)]
mod component_test;
#[cfg(test)]
mod configuration_test;
#[cfg(test)]
mod parameter_test;
#[cfg(test)]
mod traits_test;

/// Application defines a Hydra application
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Application {}

use std::collections::HashMap;

// TODO: This part is not specified in the spec b/c it is considered a runtime
// detail of Kubernetes. Need to fill this in as we go.

/// HydraStatus is the status of a Hydra Configuration object, per Kubernetes.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HydraStatus {
    pub phase: Option<String>,
    pub health: Option<HashMap<String, String>>,
}

impl HydraStatus {
    pub fn new(phase: Option<String>) -> HydraStatus {
        HydraStatus {
            phase: phase,
            health: None,
        }
    }
}

/// HydraComponentStatus is the status of a Hydra Component object, per Kubernetes.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HydraComponentStatus {
    pub phase: Option<String>,
}

impl Default for HydraStatus {
    fn default() -> Self {
        HydraStatus {
            phase: None,
            health: None,
        }
    }
}

/// Status is a convenience for an optional HydraStatus.
pub type Status = Option<HydraStatus>;

pub type ComponentStatus = Option<HydraComponentStatus>;

/// GroupVersionKind represents a fully qualified identifier for a resource type.
///
/// It is, as the name suggests, composed of three pieces of information:
/// - Group is a namespace
/// - Version is an API version
/// - Kind is the actual type marker
pub struct GroupVersionKind {
    pub group: String,
    pub version: String,
    pub kind: String,
}

/// GroupVersionKind represents a canonical name, composed of group, version, and (you guessed it) kind.
///
/// Group is a dotted name. While the specification requires at least one dot in the group, we do not enforce.
/// Version is an API version
/// Kind the name of the type
impl GroupVersionKind {
    /// Create a new GroupVersionKind from each component.
    ///
    /// This does not check the formatting of each part.
    pub fn new(group: &str, version: &str, kind: &str) -> GroupVersionKind {
        GroupVersionKind {
            group: group.into(),
            version: version.into(),
            kind: kind.into(),
        }
    }
    /// Parse a string into a GroupVersionKind.
    pub fn from_str(gvp: &str) -> Result<GroupVersionKind, Error> {
        // I suspect that this function could be made much more elegant.
        let parts: Vec<&str> = gvp.splitn(2, "/").collect();
        if parts.len() != 2 {
            return Err(err_msg("missing version and kind"));
        }

        let vk: Vec<&str> = parts.get(1).unwrap().splitn(2, ".").collect();
        if vk.len() != 2 {
            return Err(err_msg("missing kind"));
        }

        Ok(GroupVersionKind {
            group: parts.get(0).unwrap().to_string(),
            version: vk.get(0).unwrap().to_string(),
            kind: vk.get(1).unwrap().to_string(),
        })
    }
}
