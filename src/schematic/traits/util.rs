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
pub fn trait_labels(name: String, inst_name: String) -> Labels {
    let mut labels: Labels = BTreeMap::new();
    labels.insert("oam.dev/role".into(), "trait".into());
    labels.insert("app.kubernetes.io/name".to_string(), name);
    labels.insert("instance-name".to_string(), inst_name);
    labels
}

#[cfg(test)]
mod tests {
    use crate::schematic::traits::util::*;

    #[test]
    fn test_trait_labels() {
        let labels = trait_labels("name".to_string(), "inst".to_string());
        assert_eq!(
            "trait".to_string(),
            *labels.get("oam.dev/role").expect("role must be a string")
        );
        assert_eq!(
            "name".to_string(),
            *labels.get("app.kubernetes.io/name").expect("name must be a string")
        );
        assert_eq!(
            "inst".to_string(),
            *labels.get("instance-name").expect("instance-name must be a string")
        );
    }
}