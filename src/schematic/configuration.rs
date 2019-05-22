use crate::schematic::traits::TraitBinding;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    traits: Vec<TraitBinding>
}