use chrono::Local;
use clap::{App, Arg};
use env_logger;
use failure::{format_err, Error};
use futures::StreamExt;
use futures::future::TryFutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use kube::api::{ListParams, Object, ObjectList, CustomResource, WatchEvent};
use kube::runtime::Informer;
use kube::client::Client;
use log::{debug, error, info};
use std::convert::Infallible;
use std::io::Write;

use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1beta1::{
    CustomResourceDefinitionSpec as CrdSpec, CustomResourceDefinitionStatus as CrdStatus,
};
use rudr::instigator::{
    Instigator, COMPONENT_CRD, CONFIG_CRD, CONFIG_GROUP, CONFIG_VERSION, SCOPE_CRD, TRAIT_CRD,
};
use rudr::kube_event;
use rudr::schematic::{
    configuration::ApplicationConfiguration, OAMStatus,
};

const DEFAULT_NAMESPACE: &str = "default";

async fn kubeconfig() -> kube::Result<kube::config::Config> {
    // If env var is set, use in cluster config
    match std::env::var("KUBERNETES_PORT") {
        Ok(_val) => {
            info!("Loading in-cluster config");
            kube::config::Config::from_cluster_env()
        }
        Err(_e) => kube::config::Config::infer().await,
    }
}

type KubeOpsConfig = ApplicationConfiguration;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "trace");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{}:{}:{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                record.file().unwrap_or("<unknown>"),
                record.line().unwrap_or(0),
                &record.args()
            )
        })
        .init();

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

    info!("starting server");

    let top_ns = std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.into());
    let top_cfg = kubeconfig().await.expect("Load default kubeconfig");
    info!("apiserver:{}", top_cfg.cluster_url);

    // There is probably a better way to do this than to create two clones, but there is a potential
    // thread safety issue here.
    let cfg_watch = top_cfg.clone();
    let client = Client::new(top_cfg);

    precheck_crds(&client).await?;

    // Watch for configuration objects to be added, and react to those.
    let configuration_watch = tokio::spawn(async move {
        let ns = top_ns.clone();
        let client = Client::new(cfg_watch.clone());
        let resource = CustomResource::kind(CONFIG_CRD)
            .within(ns.as_str())
            .group(CONFIG_GROUP)
            .version(CONFIG_VERSION)
            .into_resource();
        //init all the existing objects at initiate, this should be done by informer
        let req = resource.list(&ListParams::default()).unwrap();
        match client.request::<ObjectList<KubeOpsConfig>>(req).await {
            Ok(cfgs) => {
                for cfg in cfgs.items {
                    let event = WatchEvent::Added(cfg);
                    if let Err(res) = handle_event(&client, event, ns.clone()).await {
                        // Log the error and continue. In the future, should probably
                        // re-queue data in some cases.
                        error!("Error processing event: {:?}", res)
                    };
                }
            }
            Err(err) => error!("Error list application configs: {:?}", err),
        }
        // This listens for new items, and then processes them as they come in.
        let informer: Informer<KubeOpsConfig> =
            Informer::new(kube::Api::all(client.clone()));
        loop {
            let mut events = informer.poll().await?.boxed();
            debug!("loop");

            // Clear out the event queue
            while let Some(event) = events.next().await {
                let event = event?;
                if let Err(res) = handle_event(&client, event, ns.clone()).await {
                    // Log the error and continue. In the future, should probably
                    // re-queue data in some cases.
                    error!("Error processing event: {:?}", res)
                };
                info!("Handled event");
            }
        }
    });
    info!("ApplicationConfiguration watcher is running");

    // Sync status will periodically sync all the configuration status from their workload.
    let sync_status = tokio::spawn(async move {
        let ns =
            std::env::var("KUBERNETES_NAMESPACE").unwrap_or_else(|_| DEFAULT_NAMESPACE.into());
        let cfg_watch = kubeconfig().await.expect("Load default kubeconfig");
        let client = Client::new(cfg_watch.clone());
        loop {
            let resource = CustomResource::kind(CONFIG_CRD)
                .within(ns.as_str())
                .group(CONFIG_GROUP)
                .version(CONFIG_VERSION)
                .into_resource();
            //get all the configuration object and sync status
            let req = resource.list(&ListParams::default()).unwrap();
            if let Ok(cfgs) = client.request::<ObjectList<KubeOpsConfig>>(req).await {
                for cfg in cfgs.items {
                    if let Err(res) = sync_status(&client, cfg, ns.clone()).await {
                        error!("Error sync status: {:?}", res)
                    };
                }
            }
            tokio::time::delay_for(std::time::Duration::from_secs(10)).await;
        }
    });

    let health_server = tokio::spawn(async move {
        let addr = metrics_addr.parse().unwrap();
        info!("Health server is running on {}", addr);
        let make_svc =
            make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_health)) });
        Server::bind(&addr)
            .serve(make_svc)
            .map_err(|e| eprintln!("health server error: {}", e))
            .await
    });
    let _ = health_server.await.unwrap();
    sync_status.await.expect("status syncer crashed");
    configuration_watch.await.unwrap()
}

async fn handle_health(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(match (_req.method(), _req.uri().path()) {
        (&Method::GET, "/health") => {
            debug!("health check");
            Response::new(Body::from("OK"))
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(""))
            .unwrap(),
    })
}

/// This takes an event off the stream and delegates it to the instigator, calling the correct verb.
async fn handle_event(
    cli: &Client,
    event: WatchEvent<KubeOpsConfig>,
    namespace: String,
) -> Result<(), Error> {
    let inst = Instigator::new(cli.clone(), namespace);
    match event {
        WatchEvent::Added(o) => {
            if let Err(err) = inst.add(o.clone()).await {
                if let Err(e) = inst.event_handler.push_event_message(
                    kube_event::Type::Warning,
                    kube_event::Info {
                        action: "create".to_string(),
                        message: format!("create config {} error", o.metadata.name.clone().unwrap()),
                        reason: format!("{}", err),
                    },
                    rudr::instigator::get_object_ref(o.clone()),
                ).await {
                    log::warn!("push event message for update err {}", e)
                }
                return Err(err);
            }
            Ok(())
        }
        WatchEvent::Modified(o) => {
            if let Err(err) = inst.modify(o.clone()).await {
                if let Err(e) = inst.event_handler.push_event_message(
                    kube_event::Type::Warning,
                    kube_event::Info {
                        action: "update".to_string(),
                        message: format!("update config {} error", o.metadata.name.clone().unwrap()),
                        reason: format!("{}", err),
                    },
                    rudr::instigator::get_object_ref(o.clone()),
                ).await {
                    log::warn!("push event message for update err {}", e)
                }
                return Err(err);
            }
            Ok(())
        }
        WatchEvent::Deleted(o) => inst.delete(o).await,
        WatchEvent::Error(ref e) => match e {
            kube::error::ErrorResponse { reason, .. } if reason == "AlreadyExists" => {
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
        _ => Ok(())
    }
}

async fn sync_status(cli: &Client, event: KubeOpsConfig, namespace: String) -> Result<(), Error> {
    let inst = Instigator::new(cli.clone(), namespace);
    inst.sync_status(event).await
}

type CrdObj = Object<CrdSpec, CrdStatus>;
async fn precheck_crds(client: &Client) -> Result<(), failure::Error> {
    let crds = vec![CONFIG_CRD, TRAIT_CRD, COMPONENT_CRD, SCOPE_CRD];
    for crd in crds.iter() {
        // FIXME: not sure how to do this
        /*
        let req = CustomResourceDefinition()
            .get(format!("{}.core.oam.dev", crd).as_str())?;
        if let Err(e) = client.request::<CrdObj>(req).await {
            error!("Error prechecking CRDs {}: {:?}", crd, e);
            return Err(failure::format_err!("Missing CRD {}", crd));
        }
        */
    }
    Ok(())
}
