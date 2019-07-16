use kube::{client::APIClient, config::Configuration};

use crate::schematic::component::Component;
use crate::workload_type::{KubeName, replicated_task::*};

use std::collections::BTreeMap;

#[test]
fn test_replicated_task_kube_name() {
    let cli = APIClient::new(mock_kube_config());

    let task = ReplicatedTask {
        name: "mytask".into(),
        component_name: "taskrunner".into(),
        instance_name: "taskinstance".into(),
        namespace: "tests".into(),
        definition: Component {
            ..Default::default()
        },
        params: BTreeMap::new(),
        client: cli,
        owner_ref: None,
        replica_count: None,
    };

    assert_eq!("taskinstance", task.kube_name().as_str());
}

#[test]
fn test_replicated_task_replica_count() {
    let cli = APIClient::new(mock_kube_config());

    let task = ReplicatedTask {
        name: "mytask".into(),
        component_name: "taskrunner".into(),
        instance_name: "taskinstance".into(),
        namespace: "tests".into(),
        definition: Component {
            ..Default::default()
        },
        params: BTreeMap::new(),
        client: cli,
        owner_ref: None,
        replica_count: Some(5),
    };

    let job = task.to_job();
    
    assert_eq!(Some(5), job.spec.unwrap().parallelism);
}

/// This mock builds a KubeConfig that will not be able to make any requests.
fn mock_kube_config() -> Configuration {
    Configuration {
        base_path: ".".into(),
        client: reqwest::Client::new(),
    }
}