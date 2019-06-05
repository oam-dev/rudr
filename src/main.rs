#[macro_use]
extern crate failure;

use kube::api::{ApiResource, Informer, Reflector, WatchEvent};
use kube::{client::APIClient, config::load_kube_config};

use scylla::instigator::Instigator;
use scylla::schematic::{component::Component, configuration::OperationalConfiguration, Status};

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
    let component_cache: Reflector<Component, Status> =
        Reflector::new(client.clone(), component_resource.clone().into())
            .expect("component reflector cannot be created");
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
        let informer: Informer<OperationalConfiguration, Status> =
            Informer::new(client.clone(), resource.clone().into())?;
        loop {
            informer.poll()?;
            println!("loop");

            // Clear out the event queue
            while let Some(event) = informer.pop() {
                if let Err(res) = handle_event(&client, event, component_cache.clone()) {
                    // Log the error and continue. In the future, should probably
                    // re-queue data in some cases.
                    println!("Error processing event: {:?}", res)
                };
                println!("Handled event");
            }
        }
    });

    // Cache all of the components.
    let component_watch = std::thread::spawn(move || loop {
        if let Err(res) = reflector.poll() {
            println!("Component polling error: {}", res);
        };
    });
    println!("Watcher is running");
    component_watch.join().expect("component watcher crashed");
    configuration_watch.join().unwrap()
}

/// This takes an event off the stream and delegates it to the instagator, calling the correct verb.
fn handle_event(
    cli: &APIClient,
    event: WatchEvent<OperationalConfiguration, Status>,
    cache: Reflector<Component, Status>,
) -> Result<(), failure::Error> {
    let inst = Instigator::new(cli.clone(), cache);
    match event {
        WatchEvent::Added(o) => inst.add(o),
        WatchEvent::Modified(o) => inst.modify(o),
        WatchEvent::Deleted(o) => inst.delete(o),
        WatchEvent::Error(e) => Err(format_err!("APIError: {:?}", e)),
    }
}
