use crate::schematic::traits::{TraitImplementation, util::*};
use crate::workload_type::{ParamMap, REPLICATED_SERVICE_NAME, REPLICATED_TASK_NAME};
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

/// A manual scaler provides a way to manually scale replicable objects.
#[derive(Clone, Debug)]
pub struct ManualScaler {
    pub name: String,
    pub instance_name: String,
    pub component_name: String,
    pub owner_ref: OwnerRefs,
    pub replica_count: i32,
}

impl ManualScaler {
    fn scale(&self, ns: &str, client: APIClient) -> TraitResult {
        // So what we need to do is lookup the right object and then modify it.
        Ok(())
    }
}

impl TraitImplementation for ManualScaler {
    fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        self.scale(ns, client)
    }
    fn modify(&self, ns: &str, client: APIClient) -> TraitResult {
        self.scale(ns, client)
    }
    fn supports_workload_type(name: &str) -> bool {
        // Only support replicated service and task right now.
        name == REPLICATED_SERVICE_NAME || name == REPLICATED_TASK_NAME
    }
}