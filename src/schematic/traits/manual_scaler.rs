use crate::schematic::traits::{util::*, TraitImplementation};
use crate::workload_type::{ParamMap, REPLICATED_SERVICE_NAME, REPLICATED_TASK_NAME, WORKER};
use k8s_openapi::api::{apps::v1 as apps, batch::v1 as batch};
use kube::client::APIClient;
use log::info;

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
        ManualScaler {
            name,
            instance_name,
            component_name,
            owner_ref,
            workload_type,
            replica_count: params
                .get("replicaCount")
                .and_then(|p| p.as_i64().and_then(|i64| Some(i64 as i32)))
                .unwrap_or(1),
        }
    }

    fn scale(&self, ns: &str, client: APIClient) -> TraitResult {
        // TODO: We probably need to watch for the deployment to be created. Or this might be unnecessary.
        std::thread::sleep(std::time::Duration::from_secs(5));

        info!("Scaling {} to {:?}", &self.name, &self.replica_count);
        // It should be a safe assumption that we can look up every job and every
        // deployment with a particular Kubernetess name and update them appropriately.
        match self.workload_type.as_str() {
            REPLICATED_SERVICE_NAME | WORKER => {
                let (req, _) = apps::Deployment::read_namespaced_deployment(
                    self.instance_name.as_str(),
                    ns,
                    Default::default(),
                )?;
                let res = client.request(req);
                if res.is_ok() {
                    let original: apps::Deployment = res.unwrap();
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
            REPLICATED_TASK_NAME => {
                // Scale jobs
                let (jobreq, _) = batch::Job::read_namespaced_job(
                    self.instance_name.as_str(),
                    ns,
                    Default::default(),
                )?;
                let jobres = client.request(jobreq);
                if jobres.is_ok() {
                    let original: batch::Job = jobres.unwrap();
                    let new_job = self.scale_job(original);

                    let (req2, _) = batch::Job::replace_namespaced_job(
                        self.instance_name.as_str(),
                        ns,
                        &new_job,
                        Default::default(),
                    )?;
                    client.request::<batch::Job>(req2)?;
                }
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
        name == REPLICATED_SERVICE_NAME || name == REPLICATED_TASK_NAME || name == WORKER
    }
}
