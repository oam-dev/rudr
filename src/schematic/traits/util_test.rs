use crate::schematic::traits::util::*;

#[test]
fn test_trait_labels() {
    let labels = trait_labels();
    assert_eq!(
        "trait".to_string(),
        *labels.get("hydra.io/role").expect("must be a string")
    );
}