use crate::workload_type::*;

struct MockWorkloadType {}

impl WorkloadType for MockWorkloadType {
    fn add(&self) -> Result<(), failure::Error> {
        Ok(())
    }
}

/// This is a canary test to make sure that modify and delete have default implementations.
#[test]
fn test_workload_type() {
    let mwlt = MockWorkloadType {};

    assert!(mwlt.modify().is_err());
    assert!(mwlt.delete().is_ok());
}
