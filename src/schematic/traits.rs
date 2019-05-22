use crate::schematic::parameter::ParameterValue;

/// Trait describes Hydra traits.
/// 
/// Hydra traits are ops-oriented "add-ons" that can be attached to Components of the appropriate workloadType.
/// For example, an autoscaler trait can attach to a workloadType (such as ReplicableService) that can be
/// scaled up and down.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trait {

}

/// A TraitBinding attaches a trait to a component.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TraitBinding {
    name: String,
    parameter_values: Option<Vec<ParameterValue>>,
}
