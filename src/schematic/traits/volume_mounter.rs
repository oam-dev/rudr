use async_trait::async_trait;
use k8s_openapi::api::core::v1 as core;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1 as meta;
use kube::client::Client;
use log::warn;
use serde_json::map::Map;

use crate::schematic::{
    component::{AccessMode, Component, SharingPolicy, Volume},
    traits::util::{OwnerRefs, TraitResult},
    traits::TraitImplementation,
};

use std::collections::BTreeMap;

/// PVCs must have a minimum size. However, the OAM model
/// does not require volume size be specified. This is the
/// default if no size is specified.
pub const DEFAULT_VOLUME_SIZE: &str = "200M";

/// The VolumeMounter trait provisions volumes that can
/// be mounted by a Component.
#[derive(Clone, Debug)]
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
    pub fn from_properties(
        name: String,
        instance_name: String,
        component_name: String,
        properties_map: Option<&Map<String, serde_json::value::Value>>,
        owner_ref: OwnerRefs,
        component: Component,
    ) -> Self {
        let instancename = instance_name.clone();
        VolumeMounter {
            name,
            component_name,
            instance_name,
            owner_ref,
            component,
            volume_name: properties_map
                        .and_then(|map| map.get("volumeName").and_then(|p| p.as_str()))
                        .unwrap_or_else( || { warn!("Unable to parse volumeName value for instance:{}. Setting it to default value:empty", instancename); "" } )
                        .to_string(),
            storage_class: properties_map
                        .and_then(|map| map.get("storageClass").and_then(|p| p.as_str()))
                        .unwrap_or_else( || { warn!("Unable to parse storageClass value for instance:{}. Setting it to default value:empty", instancename); "" } )
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
    /// Conver the volume mounter data to a PersistentVolumeClaim
    pub fn to_pvc(&self) -> core::PersistentVolumeClaim {
        let attach_to = self.find_volume();
        let size = Quantity(
            attach_to
                .and_then(|v| v.disk.as_ref().and_then(|d| Some(d.required.clone())))
                .unwrap_or_else(|| DEFAULT_VOLUME_SIZE.to_string()),
        );
        let mut reqs = BTreeMap::new();
        reqs.insert("storage".to_string(), size);
        core::PersistentVolumeClaim {
            metadata: Some(meta::ObjectMeta {
                name: Some(self.volume_name.clone()),
                labels: Some(self.labels()),
                owner_references: self.owner_ref.clone(),
                ..Default::default()
            }),
            spec: Some(core::PersistentVolumeClaimSpec {
                access_modes: Some(vec![self.mount_policy(attach_to)]),
                storage_class_name: Some(self.storage_class.clone()),
                resources: Some(core::ResourceRequirements {
                    requests: Some(reqs),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
    fn mount_policy(&self, volume: Option<&Volume>) -> String {
        volume
            .and_then(|vol| {
                Some(match vol.access_mode {
                    AccessMode::RO => "ReadOnlyMany",
                    AccessMode::RW => match vol.sharing_policy {
                        SharingPolicy::Shared => "ReadWriteMany",
                        _ => "ReadWriteOnce",
                    },
                })
            })
            .unwrap_or("ReadWriteOnce")
            .to_string()
    }

    /// Locate the volume that this mounter is supposed to attach
    fn find_volume(&self) -> Option<&Volume> {
        self.component.containers.iter().find_map(|c| {
            c.resources.volumes.as_ref().and_then(|vols| {
                vols.iter()
                    .find(|v| self.volume_name.eq_ignore_ascii_case(v.name.as_str()))
            })
        })
    }
}

#[async_trait]
impl TraitImplementation for VolumeMounter {
    /// Make sure the PVC is created before the Pod.
    /// This won't make a difference most of the time, but on fast disk provisioning operations
    /// this may help a little.
    async fn pre_add(&self, ns: &str, client: Client) -> TraitResult {
        let pvc = self.to_pvc();
        let (req, _) = core::PersistentVolumeClaim::create_namespaced_persistent_volume_claim(
            ns,
            &pvc,
            Default::default(),
        )?;
        client.request::<core::PersistentVolumeClaim>(req).await?;
        Ok(())
    }
    /// There is nothing to do on the add phase for this trait
    async fn add(&self, _ns: &str, _client: Client) -> TraitResult {
        Ok(())
    }
    async fn modify(&self, ns: &str, client: Client) -> TraitResult {
        let pvc = self.to_pvc();
        let values = serde_json::to_value(&pvc)?;
        let (req, _) = core::PersistentVolumeClaim::patch_namespaced_persistent_volume_claim(
            self.volume_name.as_str(),
            ns,
            &meta::Patch::StrategicMerge(values),
            Default::default(),
        )?;
        client.request::<core::PersistentVolumeClaim>(req).await?;
        Ok(())
    }
    async fn delete(&self, ns: &str, client: Client) -> TraitResult {
        let (req, _) = core::PersistentVolumeClaim::delete_namespaced_persistent_volume_claim(
            self.volume_name.as_str(),
            ns,
            Default::default(),
        )?;
        client.request::<core::PersistentVolumeClaim>(req).await?;
        Ok(())
    }
    async fn status(&self, ns: &str, client: Client) -> Option<BTreeMap<String, String>> {
        let pvc_name = self.volume_name.as_str();
        let key = format!("persistentvolumeclaim/{}", pvc_name);
        let mut resource = BTreeMap::new();
        let req = core::PersistentVolumeClaim::read_namespaced_persistent_volume_claim_status(
            pvc_name,
            ns,
            Default::default(),
        );
        if let Err(err) = req {
            resource.insert(key, err.to_string());
            return Some(resource);
        }

        let (raw_req, _) = req.unwrap();
        match client.request::<core::PersistentVolumeClaim>(raw_req).await {
            Ok(pvc) => {
                resource.insert(
                    key,
                    pvc.status
                        .and_then(|s| s.phase)
                        .unwrap_or_else(|| "unknown phase".to_string()),
                );
            }
            Err(e) => {
                resource.insert(key, e.to_string());
            }
        };
        Some(resource)
    }
}

#[cfg(test)]
mod test {
    use super::VolumeMounter;
    use crate::schematic::component::{
        AccessMode, Component, Container, Disk, Resources, SharingPolicy, Volume,
    };
    use crate::schematic::traits::TraitBinding;
    use serde_json::json;
    use serde_json::map::Map;

    #[test]
    fn test_from_properties_v1alpha1() {
        let component = Component {
            workload_type: "Server".into(),
            parameters: vec![],
            containers: vec![],
            workload_settings: vec![],
            ..Default::default()
        };
        let volume_mounter_alpha1_trait = TraitBinding {
            name: String::from("volume-mounter"),
            parameter_values: None,
            properties: Some(json!({
                "storageClass": "really-fast",
                "volumeName": "panda-bears"
            })),
        };

        let serialized = serde_json::to_string(&volume_mounter_alpha1_trait).unwrap();
        let deserialized_trait: TraitBinding = serde_json::from_str(&serialized).unwrap();
        let prop_map: Option<&Map<String, serde_json::value::Value>> =
            deserialized_trait.properties.as_ref().unwrap().as_object();

        let vm = VolumeMounter::from_properties(
            "my-volume-mount".to_string(),
            "instance name".to_string(),
            "component name".to_string(),
            prop_map,
            None,
            component,
        );

        assert_eq!("really-fast", vm.storage_class);
        assert_eq!("panda-bears", vm.volume_name);
    }

    fn mock_container(name: &str) -> Container {
        Container {
            name: name.to_string(),
            resources: Resources {
                volumes: Some(vec![Volume {
                    name: "panda-bears".to_string(),
                    mount_path: "/var/foo".to_string(),
                    sharing_policy: SharingPolicy::Exclusive,
                    access_mode: AccessMode::RO,
                    disk: Some(Disk {
                        required: "123M".to_string(),
                        ephemeral: false,
                    }),
                }]),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
