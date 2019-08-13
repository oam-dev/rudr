use kube::{client::APIClient, config::Configuration};

use crate::schematic::component::Component;
use crate::workload_type::{task::*, KubeName};

use std::collections::BTreeMap;

#[test]
fn test_singleton_task_kube_name() {
    let cli = APIClient::new(mock_kube_config());

    let task = SingletonTask {
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
    };

    assert_eq!("taskinstance", task.kube_name().as_str());
}

#[test]
fn test_replicated_task_kube_name() {
    let cli = APIClient::new(mock_kube_config());

    let task = Task {
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

/// This mock builds a KubeConfig that will not be able to make any requests.
fn mock_kube_config() -> Configuration {
    Configuration {
        base_path: ".".into(),
        client: reqwest::Client::new(),
    }
}
