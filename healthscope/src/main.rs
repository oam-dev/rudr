use chrono::{DateTime, Utc};
use clap::{App, Arg};
use env_logger;
use failure::{format_err, Error};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Response, Server, StatusCode};
use kube::api::{ListParams, ObjectList, RawApi};
use kube::{client::APIClient, config::incluster_config, config::load_kube_config};
use log::{debug, error, info};
use scylla::instigator::{combine_name, CONFIG_GROUP, CONFIG_VERSION};
use scylla::schematic::component_instance::KubeComponentInstance;
use scylla::schematic::scopes::health::{
    ComponentInfo, HealthScopeObject, HealthStatus, HEALTH_SCOPE_CRD, HEALTH_SCOPE_GROUP,
    HEALTH_SCOPE_VERSION,
};

const DEFAULT_NAMESPACE: &str = "default";
const DEFAULT_PROBE_INTERVAL: i64 = 30;

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

    // Watch for configuration objects to be added, and react to those.
    let health_scope_watch = std::thread::spawn(move || {
        let ns = top_ns.clone();
        let healthscope_resource = RawApi::customResource("healthscopes")
            .version("v1alpha1")
            .group("core.hydra.io")
            .within(ns.as_str());
        let client = APIClient::new(cfg_watch);
        let mut cnt = 0;
        loop {
            let req = healthscope_resource.list(&ListParams::default())?;
            match client.request::<ObjectList<HealthScopeObject>>(req) {
                Ok(health_scopes) => {
                    for scope in health_scopes.items {
                        if let Err(res) = aggregate_health(&client, scope, ns.clone()) {
                            // Log the error and continue.
                            error!("Error processing event: {:?}", res)
                        };
                    }
                }
                Err(e) => log::error!("get health scope list err {}", e),
            }
            cnt = (cnt + 1) % 10;
            if cnt == 0 {
                debug!("health scope aggregate loop running...");
            }
            //FIXME: we could change this to use an informer if we have a runtime controller queue
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    let server = std::thread::spawn(move || {
        let addr = endpoint_addr.parse().unwrap();
        info!("Server is running on {}", addr);
        hyper::rt::run(
            Server::bind(&addr)
                .serve(|| {
                    service_fn_ok(|req| {
                        let path = req.uri().path().to_owned();
                        match (req.method(), path) {
                            (&Method::GET, path) => {
                                debug!("health scope requested");
                                Response::new(Body::from(path.clone()))
                            }
                            _ => Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::from(""))
                                .unwrap(),
                        }
                    })
                })
                .map_err(|e| eprintln!("server error: {}", e)),
        );
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

    server.join().unwrap();
    health_scope_watch.join().unwrap()
}

/// This takes an event off the stream and delegates it to the instigator, calling the correct verb.
fn aggregate_health(
    client: &APIClient,
    mut event: HealthScopeObject,
    namespace: String,
) -> Result<(), Error> {
    let interval = event.spec.probe_interval.unwrap_or(DEFAULT_PROBE_INTERVAL);
    if !time_to_aggregate(event.status.clone(), interval) {
        return Ok(());
    }
    info!("start to probe instance: {}", event.metadata.name);
    match (
        event.spec.probe_method.as_str(),
        event.spec.probe_endpoint.as_str(),
    ) {
        ("kube-get", ".status") => {
            let components =
                event
                    .status
                    .and_then(|status| status.components)
                    .and_then(|mut components| {
                        for c in components.iter_mut() {
                            c.status = Some(get_health_from_component(
                                client,
                                c.clone(),
                                namespace.clone(),
                            ))
                        }
                        Some(components)
                    });
            event.status = Some(HealthStatus {
                components,
                last_aggregate_timestamp: Some(Utc::now().to_rfc3339()),
            });
            let pp = kube::api::PatchParams::default();
            let healthscope_resource = RawApi::customResource(HEALTH_SCOPE_CRD)
                .version(HEALTH_SCOPE_VERSION)
                .group(HEALTH_SCOPE_GROUP)
                .within(namespace.as_str());
            let req = healthscope_resource.patch(
                event.metadata.clone().name.as_str(),
                &pp,
                serde_json::to_vec(&event)?,
            )?;
            client.request::<HealthScopeObject>(req)?;
            Ok(())
        }
        _ => {
            return Err(format_err!(
                "unknown probe-method {} and probe_endpoint {}",
                event.spec.probe_method,
                event.spec.probe_endpoint
            ))
        }
    }
}

fn get_health_from_component(client: &APIClient, info: ComponentInfo, namespace: String) -> String {
    let name = combine_name(info.name, info.instance_name);
    let crd_req = RawApi::customResource("componentinstances")
        .group(CONFIG_GROUP)
        .version(CONFIG_VERSION)
        .within(namespace.as_str());
    let req = crd_req.get(name.as_str()).unwrap();
    let res: KubeComponentInstance = match client.request(req) {
        Ok(ins) => ins,
        Err(e) => {
            error!("get component instance failed {}", e);
            return "unhealthy".to_string();
        }
    };
    res.status.unwrap_or("unhealthy".to_string())
}

fn time_to_aggregate(status: Option<HealthStatus>, interval: i64) -> bool {
    if interval <= 0 {
        return true;
    }
    if status.is_none() || status.clone().unwrap().last_aggregate_timestamp.is_none() {
        return true;
    }
    let last_aggregate_time = status.unwrap().last_aggregate_timestamp.unwrap();
    let last_time = match DateTime::parse_from_rfc3339(last_aggregate_time.as_str()) {
        Ok(last) => last,
        Err(e) => {
            error!("parse last time err {}", e);
            return true;
        }
    };
    let sys_time = Utc::now();
    let duration = sys_time.signed_duration_since(last_time);
    if duration.num_seconds() >= interval {
        return true;
    }
    false
}

#[cfg(test)]
mod test {
    use crate::time_to_aggregate;
    use chrono::{DateTime, Duration, Utc};
    use scylla::schematic::scopes::health::HealthStatus;

    #[test]
    fn test_time_to_action() {
        assert_eq!(time_to_aggregate(None, 10), true);
        let status = Some(HealthStatus {
            last_aggregate_timestamp: None,
            ..Default::default()
        });
        assert_eq!(time_to_aggregate(status, 10), true);
        let status = Some(HealthStatus {
            last_aggregate_timestamp: Some(
                Utc::now()
                    .checked_sub_signed(Duration::seconds(11))
                    .unwrap()
                    .to_rfc3339(),
            ),
            ..Default::default()
        });
        assert_eq!(time_to_aggregate(status.clone(), 10), true);
        assert_eq!(time_to_aggregate(status.clone(), 15), false);
        assert_eq!(time_to_aggregate(status.clone(), 0), true);
    }
}
