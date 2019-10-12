use clap::{App, Arg};
use env_logger;
use failure::{format_err, Error};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Response, Server, StatusCode};
use kube::api::{Informer, ListParams, Object, ObjectList, RawApi, Reflector, WatchEvent};
use kube::{client::APIClient, config::incluster_config, config::load_kube_config};
use log::{debug, error, info};

use healthscope::instigator::Instigator;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::{
    CustomResourceDefinitionSpec as CrdSpec, CustomResourceDefinitionStatus as CrdStatus,
};
use scylla::instigator::{CONFIG_CRD, CONFIG_GROUP, CONFIG_VERSION};
use scylla::schematic::{configuration::ApplicationConfiguration, Status};

type KubeOpsConfig = Object<ApplicationConfiguration, Status>;

const DEFAULT_NAMESPACE: &str = "default";

fn kubeconfig() -> kube::Result<kube::config::Configuration> {
    // If env var is set, use in cluster config
    if std::env::var("KUBERNETES_PORT").is_ok() {
        return incluster_config();
    }
    load_kube_config()
}

fn main() -> Result<(), Error> {
    let flags = App::new("healthscope")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("metrics-addr")
                .short("m")
                .long("metrics-addr")
                .default_value(":8080")
                .help("The address the metric endpoint binds to."),
        )
        .arg(
            Arg::with_name("addr")
                .short("p")
                .long("endpoint-address")
                .default_value(":80")
                .help("The address the health scope endpoint binds to."),
        )
        .get_matches();
    let metrics_addr = "0.0.0.0".to_owned() + flags.value_of("metrics-addr").unwrap();
    let endpoint_addr = "0.0.0.0".to_owned() + flags.value_of("addr").unwrap();

    env_logger::init();
    info!("starting server");

    let top_ns = std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.into());
    let top_cfg = kubeconfig().expect("Load default kubeconfig");

    // There is probably a better way to do this than to create two clones, but there is a potential
    // thread safety issue here.
    let cfg_watch = top_cfg.clone();
    let client = APIClient::new(top_cfg);

    // Watch for configuration objects to be added, and react to those.
    let health_scope_watch = std::thread::spawn(move || {
        let ns = top_ns.clone();
        let client = APIClient::new(cfg_watch);
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

    health_scope_watch.join().unwrap()
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
        WatchEvent::Error(e) => Err(format_err!("APIError: {:?}", e)),
    }
}
