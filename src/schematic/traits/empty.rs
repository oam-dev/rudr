use async_trait::async_trait;
use crate::schematic::traits::{util::*, TraitImplementation};
use kube::client::APIClient;
use std::collections::BTreeMap;

pub struct Empty {}

#[async_trait]
impl TraitImplementation for Empty {
    fn supports_workload_type(_name: &str) -> bool {
        true
    }
    async fn add(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    async fn modify(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    async fn delete(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    async fn status(&self, _ns: &str, _client: APIClient) -> Option<BTreeMap<String, String>> {
        None
    }
}
