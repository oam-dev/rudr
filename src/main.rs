#[macro_use]
extern crate serde_derive;
extern crate failure;

use kube::api::{
    Reflector,
    ApiResource,
};
use kube::{
    client::APIClient,
    config::load_kube_config,
};

use std::time::Duration;

fn main() {
    let cfg = load_kube_config().expect("Load default kubeconfig");
    let cli = APIClient::new(cfg);
    let ns = "default";
    let state = State::new(ns, cli).expect("Create new state manager");

    let state_clone = state.clone(); // Allow it to be moved.
    let handle = std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(10));
            match state_clone.poll() {
                Ok(_) => println!("Refreshed"),
                Err(e) => {
                    println!("{}", e);
                    std::process::exit(1);
                }
            }
        }
    });
    println!("Watcher is running");
    handle.join().unwrap()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HydraConfigResource {
    name: String, // temporary until 0.3
    workload_type: String,
    os: Option<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HydraConfigStatus {
    phase: Option<String>,
}

#[derive(Clone)]
pub struct State {
    hydra_configs: Reflector<HydraConfigResource, Option<HydraConfigStatus>>,
}

impl State {
    fn new(ns: &str, client: APIClient) -> Result<Self, failure::Error>{
        let src = ApiResource {
            group: "hydra.microsoft.com".into(),
            resource: "components".into(),
            namespace: Some(ns.into()),
            //version: "v1alpha1".into(),
            version: "v1".into(),
            prefix: "apis".into(),
        };
        let hydra_configs = Reflector::new(client, src)?;
        Ok(State {hydra_configs})
    }

    fn poll(&self) -> Result<(), failure::Error> {
        let res = self.hydra_configs.poll();
        let btree = self.hydra_configs.read().unwrap();
        for (k, v) in btree.iter() {
            println!("{}: {:?}", k, v.metadata.annotations);
        }
        res
    }

    pub fn refresh(&self) -> Result<(), failure::Error> {
        self.hydra_configs.refresh()
    }
}