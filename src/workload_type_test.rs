use async_trait::async_trait;
use crate::workload_type::*;
use failure::Error;

struct MockWorkloadType {}

#[async_trait]
impl WorkloadType for MockWorkloadType {
    async fn add(&self) -> Result<(), Error> {
        Ok(())
    }
}

/// This is a canary test to make sure that modify and delete have default implementations.
#[tokio::test]
async fn test_workload_type() {
    let mwlt = MockWorkloadType {};

    assert!(mwlt.modify().await.is_err());
    assert!(mwlt.delete().await.is_ok());
}
