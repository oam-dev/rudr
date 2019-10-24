use clap::{App, Arg};
use env_logger;
use failure::{format_err, Error};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Response, Server, StatusCode};
use kube::api::{Informer, ListParams, Object, ObjectList, RawApi, Reflector, WatchEvent};
use kube::{client::APIClient, config::incluster_config, config::load_kube_config, ApiError};
use log::{debug, error, info};

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::{
    CustomResourceDefinitionSpec as CrdSpec, CustomResourceDefinitionStatus as CrdStatus,
};
use rudr::instigator::{
    Instigator, COMPONENT_CRD, CONFIG_CRD, CONFIG_GROUP, CONFIG_VERSION, SCOPE_CRD, TRAIT_CRD,
};
use rudr::schematic::{
    component::Component, configuration::ApplicationConfiguration, OAMStatus, Status,
};

const DEFAULT_NAMESPACE: &str = "default";

fn kubeconfig() -> kube::Result<kube::config::Configuration> {
    // If env var is set, use in cluster config
    if std::env::var("KUBERNETES_PORT").is_ok() {
        return incluster_config();
    }
    load_kube_config()
}

type KubeComponent = Object<Component, Status>;
type KubeOpsConfig = Object<ApplicationConfiguration, OAMStatus>;

fn main() -> Result<(), Error> {
    let flags = App::new("rudr")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("metrics-addr")
                .short("m")
                .long("metrics-addr")
                .default_value(":8080")
                .help("The address the metric endpoint binds to."),
        )
        .get_matches();
    let metrics_addr = "0.0.0.0".to_owned() + flags.value_of("metrics-addr").unwrap();

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
        let client = APIClient::new(cfg_watch.clone());
        let resource = RawApi::customResource(CONFIG_CRD)
            .within(ns.as_str())
            .group(CONFIG_GROUP)
            .version(CONFIG_VERSION);
        //init all the existing objects at initiate, this should be done by informer
        let req = resource.list(&ListParams::default()).unwrap();
        if let Ok(cfgs) = client.request::<ObjectList<KubeOpsConfig>>(req) {
            for cfg in cfgs.items {
                let event = WatchEvent::Added(cfg);
                if let Err(res) = handle_event(&client, event, ns.clone()) {
                    // Log the error and continue. In the future, should probably
                    // re-queue data in some cases.
                    error!("Error processing event: {:?}", res)
                };
            }
        }

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

    // Sync status will periodically sync all the configuration status from their workload.
    let sync_status = std::thread::spawn(move || {
        loop {
            let ns =
                std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.into());
            let cfg_watch = kubeconfig().expect("Load default kubeconfig");
            let client = APIClient::new(cfg_watch.clone());
            let resource = RawApi::customResource(CONFIG_CRD)
                .within(ns.as_str())
                .group(CONFIG_GROUP)
                .version(CONFIG_VERSION);
            //get all the configuration object and sync status
            let req = resource.list(&ListParams::default()).unwrap();
            if let Ok(cfgs) = client.request::<ObjectList<KubeOpsConfig>>(req) {
                for cfg in cfgs.items {
                    if let Err(res) = sync_status(&client, cfg, ns.clone()) {
                        error!("Error sync status: {:?}", res)
                    };
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    });

    // Cache all of the components.
    let component_watch = std::thread::spawn(move || loop {
        if let Err(res) = reflector.poll() {
            error!("Component polling error: {}", res);
        };
    });
    info!("Watcher is running");

    std::thread::spawn(move || {
        let addr = metrics_addr.parse().unwrap();
        info!("Health server is running on {}", addr);
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
    sync_status.join().expect("status syncer crashed");
    component_watch.join().expect("component watcher crashed");
    configuration_watch.join().unwrap()
}

/// This takes an event off the stream and delegates it to the instigator, calling the correct verb.
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
        WatchEvent::Error(ref e) => match e {
            ApiError { reason, .. } if reason == "AlreadyExists" => {
                // TODO: The configuration watch code above (lines: [71:108]) appears
                // to create k8s resources initially and then poll for events.
                //
                // The initial events created, perpetuate ADDED watch events which
                // we react on trying to re-create the already created resources.
                //
                // For now, as this to be an innocuous albeit annoying error displayed
                // in the logs, we just filter "AlreadyExists" errors to reduce confusion.
                Ok(())
            }
            _ => Err(format_err!("APIError: {:?}", e)),
        },
    }
}

fn sync_status(cli: &APIClient, event: KubeOpsConfig, namespace: String) -> Result<(), Error> {
    let inst = Instigator::new(cli.clone(), namespace);
    inst.sync_status(event)
}

type CrdObj = Object<CrdSpec, CrdStatus>;
fn precheck_crds(client: &APIClient) -> Result<(), failure::Error> {
    let crds = vec![CONFIG_CRD, TRAIT_CRD, COMPONENT_CRD, SCOPE_CRD];
    for crd in crds.iter() {
        let req = RawApi::v1beta1CustomResourceDefinition()
            .get(format!("{}.core.oam.dev", crd).as_str())?;
        if let Err(e) = client.request::<CrdObj>(req) {
            error!("Error prechecking CRDs {}: {}", crd, e);
            return Err(failure::format_err!("Missing CRD {}", crd));
        }
    }
    Ok(())
}
