use env_logger;
use failure::{format_err, Error};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Response, Server, StatusCode};
use kube::api::{Informer, Object, RawApi, Reflector, WatchEvent};
use kube::{client::APIClient, config::incluster_config, config::load_kube_config};
use log::{debug, error, info};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::{
    CustomResourceDefinitionSpec as CrdSpec, CustomResourceDefinitionStatus as CrdStatus,
};
use scylla::instigator::{Instigator, COMPONENT_CRD, CONFIG_CRD, CONFIG_GROUP, CONFIG_VERSION};
use scylla::schematic::{component::Component, configuration::OperationalConfiguration, Status};

const DEFAULT_NAMESPACE: &str = "default";

fn kubeconfig() -> kube::Result<kube::config::Configuration> {
    // If env var is set, use in cluster config
    if std::env::var("KUBERNETES_PORT").is_ok() {
        return incluster_config();
    }
    load_kube_config()
}

This will break the compile.

type KubeComponent = Object<Component, Status>;
type KubeOpsConfig = Object<OperationalConfiguration, Status>;

fn main() -> Result<(), Error> {
    env_logger::init();
    info!("starting server");

    let top_ns = std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.into());
    let top_cfg = kubeconfig().expect("Load default kubeconfig");

    // There is probably a better way to do this than to create two clones, but there is a potential
    // thread safety issue here.
    let cfg_watch = top_cfg.clone();
    let client = APIClient::new(top_cfg);

    precheck_crds(&client)?;

    let component_resource = RawApi::customResource(COMPONENT_CRD)
        .within(top_ns.as_str())
        .group(CONFIG_GROUP)
        .version(CONFIG_VERSION);

    let component_cache: Reflector<KubeComponent> =
        Reflector::raw(client.clone(), component_resource.clone()).timeout(10);
    let reflector = component_cache.clone();
    if let Err(err) = component_cache.init() {
        error!("Component init error: {}", err);
    }

    // Watch for configuration objects to be added, and react to those.
    let configuration_watch = std::thread::spawn(move || {
        let ns = top_ns.clone();
        let client = APIClient::new(cfg_watch);
        let resource = RawApi::customResource(CONFIG_CRD)
            .within(ns.as_str())
            .group(CONFIG_GROUP)
            .version(CONFIG_VERSION);

        // This listens for new items, and then processes them as they come in.
        let informer: Informer<KubeOpsConfig> =
            Informer::raw(client.clone(), resource.clone()).init()?;
        loop {
            informer.poll()?;
            debug!("loop");

            // Clear out the event queue
            while let Some(event) = informer.pop() {
                if let Err(res) = handle_event(&client, event, ns.clone()) {
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
        let addr = "0.0.0.0:8080".parse().unwrap();
        hyper::rt::run(
            Server::bind(&addr)
                .serve(|| {
                    service_fn_ok(|_req| match (_req.method(), _req.uri().path()) {
                        (&Method::GET, "/health") => {
                            debug!("health check");
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
) -> Result<(), Error> {
    let inst = Instigator::new(cli.clone(), namespace);
    match event {
        WatchEvent::Added(o) => inst.add(o),
        WatchEvent::Modified(o) => inst.modify(o),
        WatchEvent::Deleted(o) => inst.delete(o),
        WatchEvent::Error(e) => Err(format_err!("APIError: {:?}", e)),
    }
}

type CrdObj = Object<CrdSpec, CrdStatus>;
fn precheck_crds(client: &APIClient) -> Result<(), failure::Error> {
    let crds = vec![
        "operationalconfigurations",
        "traits",
        "componentschematics",
        "scopes",
    ];
    for crd in crds.iter() {
        let req = RawApi::v1beta1CustomResourceDefinition()
            .get(format!("{}.core.hydra.io", crd).as_str())?;
        if let Err(e) = client.request::<CrdObj>(req) {
            error!("Error prechecking CRDs: {}", e);
            return Err(failure::format_err!("Missing CRD {}", crd));
        }
    }
    Ok(())
}
