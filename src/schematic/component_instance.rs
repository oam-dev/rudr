#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInstance {
    pub traits: Option<Vec<crate::schematic::traits::TraitBinding>>,
}

/// Convenience type for Kubernetes wrapped ComponentInstance.
pub type KubeComponentInstance = kube::api::Object<ComponentInstance, kube::api::Void>;
