use crate::schematic::traits::{util::*, TraitImplementation};
use crate::workload_type::extended_workload::openfaas::KubeFaaS;
use crate::workload_type::{ParamMap, SERVER_NAME, TASK_NAME, WORKER_NAME};
use k8s_openapi::api::{apps::v1 as apps, batch::v1 as batch};
use kube::api::{PatchParams, RawApi};
use kube::client::APIClient;
use log::info;
use std::collections::BTreeMap;
use log::{warn};
use serde_json::map::Map;

/// A manual scaler provides a way to manually scale replicable objects.
#[derive(Clone, Debug)]
pub struct ManualScaler {
    pub name: String,
    pub instance_name: String,
    pub component_name: String,
    pub owner_ref: OwnerRefs,
    pub replica_count: i32,
    pub workload_type: String,
}

impl ManualScaler {
    pub fn from_params(
        name: String,
        instance_name: String,
        component_name: String,
        params: ParamMap,
        owner_ref: OwnerRefs,
        workload_type: String,
    ) -> ManualScaler {
        log::debug!("params: {:?}", &params);
        let instancename = instance_name.clone();
        ManualScaler {
            name,
            instance_name,
            component_name,
            owner_ref,
            workload_type,
            replica_count: params
                .get("replicaCount")
                .and_then(|p| p.as_i64().and_then(|i64| Some(i64 as i32)))
                .unwrap_or_else(|| params
                    .get("replicaCount")
                    .and_then(|p| p.as_str().and_then(|pstr| Some(pstr.parse::<i32>().unwrap_or_else(|_| { 
                            warn!("replicaCount value is provided as string instead of 'int' for the instance:{}. Setting it to default:1.", instancename); 1 
                        }))))
                    .unwrap_or_else( || { warn!("Unable to parse replicaCount value for instance:{}. Setting it to default:1.", instancename); 1} )
                   ),
        }
    }
    pub fn from_properties(
        name: String,
        instance_name: String,
        component_name: String,
        properties_map: Option<&Map<String, serde_json::value::Value>>,
        owner_ref: OwnerRefs,
        workload_type: String,
    ) -> ManualScaler {
        let instancename = instance_name.clone();
        ManualScaler {
            name,
            instance_name,
            component_name,
            owner_ref,
            workload_type,
            replica_count: properties_map
                        .and_then(|map| map.get("replicaCount").and_then(|p| p.as_i64().and_then(|p64| Some(p64 as i32)))
                        ).unwrap_or_else( || { warn!("Unable to parse replicaCount value for instance:{}. Setting it to default value:80", instancename); 1}),
        }
    }
    fn scale(&self, ns: &str, client: APIClient) -> TraitResult {
        // TODO: We probably need to watch for the deployment to be created. Or this might be unnecessary.
        std::thread::sleep(std::time::Duration::from_secs(5));

        info!("Scaling {} to {:?}", &self.name, &self.replica_count);
        // It should be a safe assumption that we can look up every job and every
        // deployment with a particular Kubernetess name and update them appropriately.
        match self.workload_type.as_str() {
            SERVER_NAME | WORKER_NAME => {
                let (req, _) = apps::Deployment::read_namespaced_deployment(
                    self.instance_name.as_str(),
                    ns,
                    Default::default(),
                )?;
                let res = client.request(req);
                if let Ok(original) = res {
                    let dep = self.scale_deployment(original);

                    let (req2, _) = apps::Deployment::replace_namespaced_deployment(
                        self.instance_name.as_str(),
                        ns,
                        &dep,
                        Default::default(),
                    )?;
                    client.request::<apps::Deployment>(req2)?;
                }
                Ok(())
            }
            TASK_NAME => {
                // Scale jobs
                let (jobreq, _) = batch::Job::read_namespaced_job(
                    self.instance_name.as_str(),
                    ns,
                    Default::default(),
                )?;
                if let Ok(original) = client.request(jobreq) {
                    let new_job = self.scale_job(original);
                    let (req2, _) = batch::Job::replace_namespaced_job(
                        self.instance_name.as_str(),
                        ns,
                        &new_job,
                        Default::default(),
                    )?;
                    client.request::<batch::Job>(req2)?;
                };
                Ok(())
            }
            crate::workload_type::extended_workload::openfaas::OPENFAAS => {
                // Scale openfaas workload
                let faas_resource = RawApi::customResource("functions")
                    .version("v1alpha2")
                    .group("openfaas.com")
                    .within(ns);
                let faas_req = faas_resource.get(self.instance_name.clone().as_str())?;
                let mut openfaas: KubeFaaS = client.request(faas_req)?;
                let mut labels = openfaas.metadata.labels.clone();
                labels.insert("com.openfaas.scale.min".to_string(),self.replica_count.to_string());
                openfaas.metadata.labels = labels;
                let faas_req = faas_resource.patch(
                    self.instance_name.clone().as_str(),
                    &PatchParams::default(),
                    serde_json::to_vec(&openfaas)?,
                )?;
                let openfaas: KubeFaaS = client.request(faas_req)?;
                info!(
                    "openfass function {} was scaled to {}",
                    openfaas.metadata.name,
                    self.replica_count.to_string(),
                );
                Ok(())
            }
            _ => {
                info!("Unsupported workload type: {}", self.workload_type.as_str());
                Ok(())
            }
        }
    }

    /// Scale a deployment
    ///
    /// This takes a base deployment and returns a new deployment with the replica count set.
    pub fn scale_deployment(&self, deployment: apps::Deployment) -> apps::Deployment {
        let new_spec = apps::DeploymentSpec {
            replicas: Some(self.replica_count),
            ..deployment.spec.unwrap()
        };

        apps::Deployment {
            spec: Some(new_spec),
            metadata: deployment.metadata.clone(),
            ..Default::default()
        }
    }

    pub fn scale_job(&self, job: batch::Job) -> batch::Job {
        batch::Job {
            spec: Some(batch::JobSpec {
                parallelism: Some(self.replica_count),
                ..job.spec.unwrap()
            }),
            metadata: job.metadata.clone(),
            ..Default::default()
        }
    }
}

impl TraitImplementation for ManualScaler {
    fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        self.scale(ns, client)
    }
    fn modify(&self, ns: &str, client: APIClient) -> TraitResult {
        self.scale(ns, client)
    }
    fn delete(&self, _ns: &str, _client: APIClient) -> TraitResult {
        Ok(())
    }
    fn supports_workload_type(name: &str) -> bool {
        // Only support replicated service and task right now.
        name == SERVER_NAME || name == TASK_NAME || name == WORKER_NAME
    }
    fn status(&self, _ns: &str, _client: APIClient) -> Option<BTreeMap<String, String>> {
        None
    }
}
