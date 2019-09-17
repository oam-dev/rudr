use crate::schematic::traits::{util::*, TraitImplementation};
use kube::client::APIClient;

pub struct Empty {}

impl TraitImplementation for Empty {
    fn supports_workload_type(_name: &str) -> bool {
        true
    }
    fn add(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    fn modify(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    fn delete(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
}
