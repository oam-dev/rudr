use k8s_openapi::api::core::v1 as core;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::APIClient;

use crate::schematic::{
    component::Component,
    traits::util::{OwnerRefs, TraitResult},
    traits::TraitImplementation,
};
use crate::workload_type::ParamMap;

use std::collections::BTreeMap;

/// The VolumeMounter trait provisions volumes that can
/// be mounted by a Component.
pub struct VolumeMounter {
    /// The app configuration name
    pub name: String,
    /// The instance name for this component
    pub instance_name: String,
    /// The component name
    pub component_name: String,
    /// The owner reference (usually of the component instance).
    /// This should be attached to any Kubernetes resources that this trait creates.
    pub owner_ref: OwnerRefs,
    /// The component that we are attaching to
    pub component: Component,
    /// The name
    pub volume_name: String,
    /// The name of the storage class to which this will derive a PVC
    pub storage_class: String,
}

impl VolumeMounter {
    pub fn from_params(
        name: String,
        instance_name: String,
        component_name: String,
        params: ParamMap,
        owner_ref: OwnerRefs,
        component: Component,
    ) -> Self {
        VolumeMounter {
            name,
            component_name,
            instance_name,
            owner_ref,
            component,
            volume_name: params
                .get("volumeName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            storage_class: params
                .get("storageClass")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }
    }
    fn labels(&self) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.name.clone());
        labels.insert("component-name".to_string(), self.component_name.clone());
        labels.insert("instance-name".to_string(), self.instance_name.clone());
        labels.insert("trait".to_string(), "volume-mounter".to_string());
        labels
    }
    pub fn to_pvc(&self) -> core::PersistentVolumeClaim {
        core::PersistentVolumeClaim {
            metadata: Some(meta::ObjectMeta {
                name: Some(self.volume_name.clone()),
                labels: Some(self.labels()),
                owner_references: self.owner_ref.clone(),
                ..Default::default()
            }),
            spec: Some(core::PersistentVolumeClaimSpec {
                access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                storage_class_name: Some(self.storage_class.clone()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl TraitImplementation for VolumeMounter {
    fn add(&self, ns: &str, client: APIClient) -> TraitResult {
        let pvc = self.to_pvc();
        let (req, _) = core::PersistentVolumeClaim::create_namespaced_persistent_volume_claim(
            ns,
            &pvc,
            Default::default(),
        )?;
        client.request::<core::PersistentVolumeClaim>(req)?;
        Ok(())
    }
    fn modify(&self, ns: &str, client: APIClient) -> TraitResult {
        let pvc = self.to_pvc();
        let values = serde_json::to_value(&pvc)?;
        let (req, _) = core::PersistentVolumeClaim::patch_namespaced_persistent_volume_claim(
            self.volume_name.as_str(),
            ns,
            &meta::Patch::StrategicMerge(values),
            Default::default(),
        )?;
        client.request::<core::PersistentVolumeClaim>(req)?;
        Ok(())
    }
    fn delete(&self, ns: &str, client: APIClient) -> TraitResult {
        let (req, _) = core::PersistentVolumeClaim::delete_namespaced_persistent_volume_claim(
            self.volume_name.as_str(),
            ns,
            Default::default(),
        )?;
        client.request::<core::PersistentVolumeClaim>(req)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::VolumeMounter;
    use crate::schematic::component::Component;
    use crate::workload_type::ParamMap;
    #[test]
    fn test_from_params() {
        let component = Component {
            ..Default::default()
        };
        let mut params = ParamMap::new();
        params.insert("storageClass".into(), serde_json::json!("really-fast"));
        params.insert("volumeName".into(), serde_json::json!("panda-bears"));
        let vm = VolumeMounter::from_params(
            "name".to_string(),
            "instance name".to_string(),
            "component name".to_string(),
            params,
            None,
            component,
        );

        assert_eq!("really-fast", vm.storage_class);
        assert_eq!("panda-bears", vm.volume_name);
    }

    #[test]
    fn test_to_pvc() {
        let component = Component {
            workload_type: "Server".into(),
            parameters: vec![],
            containers: vec![],
            workload_settings: vec![],
            ..Default::default()
        };
        let mut params = ParamMap::new();
        params.insert("storageClass".into(), serde_json::json!("really-fast"));
        params.insert("volumeName".into(), serde_json::json!("panda-bears"));
        let pvc = VolumeMounter::from_params(
            "name".to_string(),
            "instance name".to_string(),
            "component name".to_string(),
            params,
            None,
            component,
        )
        .to_pvc();

        assert_eq!(
            "panda-bears",
            pvc.metadata.expect("metadata").name.expect("name")
        )
    }
}
