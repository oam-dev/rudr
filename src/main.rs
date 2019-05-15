#[macro_use]
extern crate serde_derive;
extern crate failure;

use kube::api::{
    ApiResource,
    Informer,
    Reflector,
    WatchEvent,
};
use kube::{
    client::APIClient,
    config::load_kube_config,
};

fn main() -> Result<(), failure::Error> {
    let handle = std::thread::spawn(move || {
        let ns = "default";
        let cfg = load_kube_config().expect("Load default kubeconfig");
        let client = APIClient::new(cfg);
        let resource = ApiResource {
            group: "hydra.microsoft.com".into(),
            resource: "components".into(),
            namespace: Some(ns.into()),
            version: "v1".into(), //"v1alpha1".into(),
            ..Default::default()
        };
        // This lists all the stuff that is there already
        let reflector: Reflector<HydraConfigResource, Option<HydraConfigStatus>> = Reflector::new(client.clone(), resource.clone().into())?;
        reflector.poll()?;
        reflector.read()?.into_iter().for_each(|(name, crd)| {
            println!("Existing {}: {:?}", name, crd.spec)
        });

        // This listens for new items, and then processes them as they come in.
        let informer: Informer<HydraConfigResource, Option<HydraConfigStatus>> = Informer::new(client.clone(), resource.clone().into())?;
        loop {
            informer.poll()?;

            // Clear out the event queue
            while let Some(event) = informer.pop() {
                handle_event(&client, event)?;
            }
            
        }
    });
    println!("Watcher is running");
    handle.join().unwrap()
}

fn handle_event(_cli: &APIClient, event: WatchEvent<HydraConfigResource, Option<HydraConfigStatus>>) -> Result<(), failure::Error> {
    match event {
        WatchEvent::Added(o) => println!("Added {}", o.metadata.name),
        WatchEvent::Modified(o) => println!("Updated {}", o.metadata.name),
        WatchEvent::Deleted(o) => println!("Deleted {}", o.metadata.name),
        WatchEvent::Error(e) => println!("Error: {:?}", e),
    }
    Ok(())
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