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

use scylla::schematic::{
    configuration::Configuration,
    component::Component,
    Status,
};
use scylla::instigator::Instigator;

// Cache is a reference counted locking BTree map for caching components.
//type ComponentCache = std::sync::Arc<std::sync::RwLock<std::collections::BTreeMap<String, Component>>>;

fn main() -> Result<(), failure::Error> {
    let top_ns = "default";
    let top_cfg = load_kube_config().expect("Load default kubeconfig");

    // There is probably a better way to do this than to create two clones, but there is a potential
    // thread safety issue here.
    let cfg_watch = top_cfg.clone();
    let client = APIClient::new(top_cfg);

    let component_resource = ApiResource {
        group: "core.hydra.io".into(),
        resource: "components".into(),
        namespace: Some(top_ns.into()),
        version: "v1alpha1".into(),
        ..Default::default()
    };
    let component_cache: Reflector<Component, Status> = Reflector::new(client.clone(), component_resource.clone().into()).expect("component reflector cannot be created");
    let reflector = component_cache.clone();

    // Watch for configuration objects to be added, and react to those. 
    let configuration_watch = std::thread::spawn(move || {
        let ns = top_ns;
        
        let client = APIClient::new(cfg_watch);
        let resource = ApiResource {
            group: "core.hydra.io".into(),
            resource: "configurations".into(),
            namespace: Some(ns.into()),
            version: "v1alpha1".into(),
            ..Default::default()
        };

        // This listens for new items, and then processes them as they come in.
        let informer: Informer<Configuration, Status> = Informer::new(client.clone(), resource.clone().into())?;
        loop {
            informer.poll()?;

            // Clear out the event queue
            while let Some(event) = informer.pop() {
                handle_event(&client, event, component_cache.clone())?;
            }
            
        }
    });

    // Cache all of the components.
    let component_watch = std::thread::spawn(move || {
        loop {
            let res = reflector.poll();
            if res.is_err() {
                println!("Component polling error: {}", res.unwrap_err());
                continue;
            }
        }
    });
    println!("Watcher is running");
    component_watch.join().expect("component watcher crashed");
    configuration_watch.join().unwrap()
}

fn handle_event(cli: &APIClient, event: WatchEvent<Configuration, Status>, cache: Reflector<Component, Status>) -> Result<(), failure::Error> {
    let inst = Instigator::new(cli.clone(), cache);
    match event {
        WatchEvent::Added(o) => {
            let res = inst.add(o);
            if res.is_err() {
                println!("{}", res.unwrap_err());
            }
        },
        WatchEvent::Modified(o) => println!("Updated {}", o.metadata.name),
        WatchEvent::Deleted(o) => println!("Deleted {}", o.metadata.name),
        WatchEvent::Error(e) => println!("Error: {:?}", e),
    }
    Ok(())
}