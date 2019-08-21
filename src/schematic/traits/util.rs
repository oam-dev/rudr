use failure::Error;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use std::collections::BTreeMap;

/// Alias for trait results.
pub type TraitResult = Result<(), Error>;
/// Alias for a vector of owner references.
pub type OwnerRefs = Option<Vec<meta::OwnerReference>>;
/// Alias for a map of labels.
type Labels = BTreeMap<String, String>;

/// Generate the common labels for a trait.
pub fn trait_labels() -> Labels {
    let mut labels: Labels = BTreeMap::new();
    labels.insert("hydra.io/role".into(), "trait".into());
    labels
}
