use crate::workload_type::{
    workload_builder::WorkloadMetadata, InstigatorResult, KubeName, WorkloadType,
};

pub struct ReplicatedWorker {
    pub meta: WorkloadMetadata,
    pub replica_count: Option<i32>,
}

impl KubeName for ReplicatedWorker {
    fn kube_name(&self) -> String {
        self.meta.kube_name()
    }
}

impl WorkloadType for ReplicatedWorker {
    fn add(&self) -> InstigatorResult {
        //pre create config_map
        self.meta.create_config_maps("replicated-worker")?;
        self.meta.create_deployment("replicated-worker")?;
        Ok(())
    }
    fn modify(&self) -> InstigatorResult {
        //TODO update config_map
        self.meta.update_deployment("replicated-worker")
    }
    fn delete(&self) -> InstigatorResult {
        self.meta.delete_deployment()
    }
}

/// Task represents a non-daemon process.
///
/// It is currently implemented as a Kubernetes Job.
pub struct SingletonWorker {
    pub meta: WorkloadMetadata,
}

impl KubeName for SingletonWorker {
    fn kube_name(&self) -> String {
        self.meta.instance_name.to_string()
    }
}

impl WorkloadType for SingletonWorker {
    fn add(&self) -> InstigatorResult {
        //pre create config_map
        self.meta.create_config_maps("singleton-worker")?;
        self.meta.create_deployment("singleton-worker")?;
        Ok(())
    }
    fn modify(&self) -> InstigatorResult {
        //TODO update config_map
        self.meta.update_deployment("singleton-worker")
    }
    fn delete(&self) -> InstigatorResult {
        self.meta.delete_deployment()
    }
}

#[cfg(test)]
mod test {
    use kube::{client::APIClient, config::Configuration};

    use crate::schematic::component::Component;
    use crate::workload_type::{worker::*, workload_builder::WorkloadMetadata, KubeName};

    use std::collections::BTreeMap;

    #[test]
    fn test_singleton_worker_kube_name() {
        let cli = APIClient::new(mock_kube_config());

        let wrkr = SingletonWorker {
            meta: WorkloadMetadata {
                name: "mytask".into(),
                component_name: "workermcworkyface".into(),
                instance_name: "workerinst".into(),
                namespace: "tests".into(),
                definition: Component {
                    ..Default::default()
                },
                params: BTreeMap::new(),
                client: cli,
                annotations: None,
                owner_ref: None,
            },
        };

        assert_eq!("workerinst", wrkr.kube_name().as_str());
    }

    #[test]
    fn test_replicated_worker_kube_name() {
        let cli = APIClient::new(mock_kube_config());
        let mut annotations = BTreeMap::new();
        annotations.insert("annotation1".to_string(), "value".to_string());

        let wrkr = ReplicatedWorker {
            meta: WorkloadMetadata {
                name: "mytask".into(),
                component_name: "workerbee".into(),
                instance_name: "workerinst".into(),
                namespace: "tests".into(),
                definition: Component {
                    ..Default::default()
                },
                annotations: Some(annotations),
                params: BTreeMap::new(),
                client: cli,
                owner_ref: None,
            },
            replica_count: Some(1),
        };

        assert_eq!("workerinst", wrkr.kube_name().as_str());
    }

    /// This mock builds a KubeConfig that will not be able to make any requests.
    fn mock_kube_config() -> Configuration {
        Configuration {
            base_path: ".".into(),
            client: reqwest::Client::new(),
        }
    }

}
