#[macro_use]
extern crate failure;
extern crate hyper;

use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Response, Server, StatusCode};

use kube::api::{Informer, Object, RawApi, Reflector, WatchEvent};
use kube::{client::APIClient, config::incluster_config, config::load_kube_config};

use scylla::instigator::Instigator;
use scylla::schematic::{component::Component, configuration::OperationalConfiguration, Status};

use log::{error, info};

fn kubeconfig() -> Result<kube::config::Configuration, failure::Error> {
    // If env var is set, use in cluster config
    if std::env::var("KUBERNETES_PORT").is_ok() {
        return incluster_config();
    }
    load_kube_config()
}

type KubeComponent = Object<Component, Status>;
type KubeOpsConfig = Object<OperationalConfiguration, Status>;

fn main() -> Result<(), failure::Error> {
    env_logger::init();

    let top_ns = "default";
    let top_cfg = kubeconfig().expect("Load default kubeconfig");

    // There is probably a better way to do this than to create two clones, but there is a potential
    // thread safety issue here.
    let cfg_watch = top_cfg.clone();
    let client = APIClient::new(top_cfg);

    let component_resource = RawApi::customResource("components")
        .within(top_ns.into())
        .group("core.hydra.io".into())
        .version("v1alpha1");

    let component_cache: Reflector<KubeComponent> =
        Reflector::raw(client.clone(), component_resource.clone())
            .timeout(10)
            .init()?;
    let reflector = component_cache.clone();

    // Watch for configuration objects to be added, and react to those.
    let configuration_watch = std::thread::spawn(move || {
        let ns = top_ns;
        let client = APIClient::new(cfg_watch);
        let resource = RawApi::customResource("configurations")
            .within(top_ns.into())
            .group("core.hydra.io".into())
            .version("v1alpha1");

        // This listens for new items, and then processes them as they come in.
        let informer: Informer<KubeOpsConfig> =
            Informer::raw(client.clone(), resource.clone().into()).init()?;
        loop {
            informer.poll()?;
            info!("loop");

            // Clear out the event queue
            while let Some(event) = informer.pop() {
                if let Err(res) = handle_event(&client, event, ns.into()) {
                    // Log the error and continue. In the future, should probably
                    // re-queue data in some cases.
                    error!("Error processing event: {:?}", res)
                };
                info!("Handled event");
            }
        }
    });

    // Cache all of the components.
    let component_watch = std::thread::spawn(move || loop {
        if let Err(res) = reflector.poll() {
            error!("Component polling error: {}", res);
        };
    });
    info!("Watcher is running");

    std::thread::spawn(|| {
        let addr = "127.0.0.1:8080".parse().unwrap();
        hyper::rt::run(
            Server::bind(&addr)
                .serve(|| {
                    service_fn_ok(|_req| match (_req.method(), _req.uri().path()) {
                        (&Method::GET, "/health") => {
                            info!("health check");
                            Response::new(Body::from("OK"))
                        }
                        _ => Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from(""))
                            .unwrap(),
                    })
                })
                .map_err(|e| eprintln!("health server error: {}", e)),
        );
    })
    .join()
    .unwrap();

    component_watch.join().expect("component watcher crashed");
    configuration_watch.join().unwrap()
}

/// This takes an event off the stream and delegates it to the instagator, calling the correct verb.
fn handle_event(
    cli: &APIClient,
    event: WatchEvent<KubeOpsConfig>,
    namespace: String,
) -> Result<(), failure::Error> {
    let inst = Instigator::new(cli.clone(), namespace);
    match event {
        WatchEvent::Added(o) => inst.add(o),
        WatchEvent::Modified(o) => inst.modify(o),
        WatchEvent::Deleted(o) => inst.delete(o),
        WatchEvent::Error(e) => Err(format_err!("APIError: {:?}", e)),
    }
}
