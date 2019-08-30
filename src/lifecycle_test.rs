use crate::lifecycle::Phase;

#[test]
fn test_lifecycle_phase() {
    let mut phase = Phase::Modify;
    assert_eq!("Modify", phase.to_string());
    phase = Phase::Delete;
    assert_eq!("Delete", phase.to_string());
}
