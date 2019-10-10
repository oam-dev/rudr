use k8s_openapi::api::core::v1 as core;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;
use kube::api::{RawApi, Object, PostParams, PatchParams};

use crate::schematic::{
    component::{AccessMode, Component, SharingPolicy, Volume},
    traits::util::{OwnerRefs, TraitResult},
    traits::TraitImplementation,
};
use crate::workload_type::ParamMap;

use std::collections::BTreeMap;


// Kubernetes spec for TrafficSplit.
#[derive(Deserialize, Serialize, Clone)]
struct TrafficSplitSpec {
    service: String,
    backends: Vec<Backend>,
}
// Kubernetes structure for the backend members for a split.
#[derive(Deserialize, Serialize, Clone)]
struct Backend {
    service: String,
    weight: i64,
}
// Provides access to the status from the SMI implementation
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
struct TrafficSplitStatus {}

/// A TrafficSplit custom resource for Kubernetes
type TrafficSplitCR = Object<TrafficSplitSpec, TrafficSplitStatus>;


/// The OAM Trait for SMI traffic splitters.
pub struct TrafficSplit {
        /// The app configuration name
    pub name: String,
    /// The instance name for this component
    pub instance_name: String,
    /// The component name
    pub component_name: String,
    /// The owner reference (usually of the component instance).
    /// This should be attached to any Kubernetes resources that this trait creates.
    pub owner_ref: OwnerRefs,
    /// The component that we are attaching to
    //pub component: Component,
    /// The name of the traffic splitter that this should join.
    pub splitter_name: String,
    /// The split weight. Defaults to 0
    pub weight: i64
}

impl TrafficSplit {
    pub fn from_params(
        name: String,
        instance_name: String,
        component_name: String,
        params: ParamMap,
        owner_ref: OwnerRefs,
        //component: Component,
    ) -> Self {
        TrafficSplit {
            name,
            component_name,
            instance_name,
            owner_ref,
            //component,
            splitter_name: params
                .get("splitterName")
                .and_then(|v| v.as_str())
                .unwrap_or("fooooo")
                .to_string(),
            weight: params
                .get("weight")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
        }
    }
    fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.name.clone());
        labels.insert("component-name".to_string(), self.component_name.clone());
        labels.insert("instance-name".to_string(), self.instance_name.clone());
        labels.insert("trait".to_string(), "traffic-split".to_string());
        labels
    }

    /// Create or update a splitter.
    /// 
    /// For both add and modify, it is possible to have a situation where the
    /// splitter exists and needs to be created, as well as a situation where the
    /// splitter does not exist.
    fn create_or_update_splitter(&self, ns: &str, client: APIClient) -> TraitResult {
        info!("Splitter trait for {} with weight {}", self.splitter_name.as_str(), self.weight);
        let splitter = RawApi::customResource("trafficsplits").version("v1alpha1").group("split.smi-spec.io").within(ns);
        // First, see if splitter already exists
        match client.request::<TrafficSplitCR>(splitter.get(self.splitter_name.as_str())?) {
            Ok(current) => {
                // If found, patch it.
                let mut backends = current.spec.backends.clone();
                backends.push(Backend{
                    service: self.instance_name.clone(),
                    weight: self.weight,
                });

                // TODO: How should we handle owner references? Right now, the one that
                // creates the group owns the group.

                let modified_def = json!({
                    "spec": {
                        "backends": backends
                    }
                });
                let pp = PatchParams::default();
                let patch = serde_json::to_vec(&modified_def)?;
                let req = splitter.patch(self.splitter_name.as_str(), &pp, patch)?;
                let res = client.request::<TrafficSplitCR>(req)?;
                info!("Patched TrafficSplitter");
                //debug!("TrafficSplitter: {:?}", res);
            },
            // We want to catch the case where an error comes back saying the resource does not exist.
            //Err(ref e) if e.api_error().map_or(false, |err| err.code == 404 ) => {
            Err(e) => {
                //info!("Error code: {}", e.api_error().unwrap().code);
                // If not found, create it
                log::info!("found no splitter to modify");
                let splitter_def = json!({
                    "apiVersion": "split.smi-spec.io/v1alpha1",
                    "kind": "TrafficSplit",
                    "metadata": {
                        "name": self.splitter_name.as_str(),
                        "labels": self.labels(),
                        "ownerReferences": self.owner_ref
                    },
                    "spec": {
                        "service": self.splitter_name.as_str(),
                        "backends": [
                            {
                                "service": self.instance_name.as_str(),
                                "weight": self.weight,
                            }
                        ]
                    }
                    
                });

                let pp = PostParams::default();
                let req = splitter.create(&pp, serde_json::to_vec(&splitter_def)?)?;
                let res = client.request::<TrafficSplitCR>(req)?;
                info!("Created new TrafficSplit {} for {}", self.splitter_name.clone(), self.instance_name.clone());
            }
        }
        Ok(())
    }
}

impl TraitImplementation for TrafficSplit {
 fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        self.create_or_update_splitter(ns, client)
    }
    fn modify(&self, ns: &str, client: APIClient) -> TraitResult {
        self.create_or_update_splitter(ns, client)
    }
    fn delete(&self, _ns: &str, _client: APIClient) -> TraitResult {
        // Right now, this is a no-op, since we don't want the instance to delete
        // what is a shared resource.
        Ok(())
    }
    fn status(&self, ns: &str, client: APIClient) -> Option<BTreeMap<String, String>> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::TrafficSplit;
    use crate::workload_type::ParamMap;
    #[test]
    fn test_from_params() {
        let mut params = ParamMap::new();
        params.insert("splitterName".into(), serde_json::json!("my-splitter"));
        params.insert("weight".into(), serde_json::json!(10));
        params.insert("luftBalloons".into(), serde_json::json!(99));
        let vm = TrafficSplit::from_params(
            "name".to_string(),
            "instance name".to_string(),
            "component name".to_string(),
            params,
            None,
        );

        assert_eq!("my-splitter", vm.splitter_name);
        assert_eq!(10, vm.weight);
    }
}