use crate::schematic::traits::{util::*, TraitImplementation};
use crate::workload_type::{ParamMap, SERVER_NAME, TASK_NAME, WORKER_NAME};
use k8s_openapi::api::{apps::v1 as apps, batch::v1 as batch};
use kube::client::APIClient;
use log::info;
use std::collections::BTreeMap;
use log::{warn};

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

#[cfg(test)]
mod test {
    use k8s_openapi::api::{apps::v1 as apps, batch::v1 as batch};

    use crate::{
        schematic::traits::{manual_scaler::ManualScaler, TraitImplementation},
        workload_type::{SERVER_NAME, SINGLETON_SERVER_NAME, SINGLETON_TASK_NAME, TASK_NAME},
    };

    #[test]
    fn test_manual_scaler_workload_types() {
        let matches = vec![SERVER_NAME, TASK_NAME];
        for m in matches {
            assert!(ManualScaler::supports_workload_type(m));
        }
        let no_matches = vec![SINGLETON_TASK_NAME, SINGLETON_SERVER_NAME];
        for m in no_matches {
            assert!(!ManualScaler::supports_workload_type(m));
        }
    }

    #[test]
    fn test_scale_deployment() {
        let first = apps::Deployment {
            spec: Some(apps::DeploymentSpec {
                replicas: Some(5),
                ..Default::default()
            }),
            ..Default::default()
        };
        let ms = ManualScaler {
            name: "name".into(),
            instance_name: "inst_name".into(),
            component_name: "comp_name".into(),
            owner_ref: None,
            replica_count: 9,
            workload_type: SERVER_NAME.into(),
        };
        let second = ms.scale_deployment(first);
        assert_eq!(Some(9), second.spec.expect("spec is required").replicas);
    }

    #[test]
    fn test_scale_job() {
        let first = batch::Job {
            spec: Some(batch::JobSpec {
                parallelism: Some(5),
                ..Default::default()
            }),
            ..Default::default()
        };
        let ms = ManualScaler {
            name: "name".into(),
            instance_name: "inst_name".into(),
            component_name: "comp_name".into(),
            owner_ref: None,
            replica_count: 9,
            workload_type: TASK_NAME.into(),
        };
        let second = ms.scale_job(first);
        assert_eq!(Some(9), second.spec.expect("spec is required").parallelism);
    }
}
