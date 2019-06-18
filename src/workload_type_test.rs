use kube::{client::APIClient, config::load_kube_config};

use crate::workload_type::*;
use crate::schematic::component::Component;

use std::collections::BTreeMap;

#[test]
fn test_singleton_kube_name() {
    let kcfg = load_kube_config().expect("load kubeconfig");
    let cli = APIClient::new(kcfg);

    let sing = Singleton{
        name: "de".into(),
        component_name: "hydrate".into(),
        namespace: "tests".into(),
        definition: Component {
            ..Default::default()
        },
        params: BTreeMap::new(),
        client: cli,
    
    };

    assert_eq!("de-hydrate", sing.kube_name().as_str());
}

#[test]
fn test_replicated_service_kube_name() {
    let kcfg = load_kube_config().expect("load kubeconfig");
    let cli = APIClient::new(kcfg);

    let rs = ReplicatedService{
        name: "de".into(),
        component_name: "hydrate".into(),
        namespace: "tests".into(),
        definition: Component {
            ..Default::default()
        },
        params: BTreeMap::new(),
        client: cli,
    };

    assert_eq!("de-hydrate", rs.kube_name().as_str());  
}