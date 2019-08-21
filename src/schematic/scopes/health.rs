use crate::schematic::parameter::ParameterValue;
use serde_json::json;

/// Health
#[derive(Clone, Debug)]
pub struct Health {
    pub name: String,
    pub health_threshold_percentage: f64, //The % of healthy components required to upgrade scope

    pub allow_component_overlap: bool,
}

impl Health {
    pub fn from_params(name: String, params: Option<Vec<ParameterValue>>) -> Self {
        // Right now, we're relying on the higher level validation logic to validate types.
        let mut threshold = 100.0;
        for p in params.unwrap() {
            if p.name == "health_threshold_percentage" {
                threshold = p.value.unwrap_or(json!(100.0)).as_f64().unwrap();
                break;
            }
        }
        Health {
            name,
            health_threshold_percentage: threshold,
            allow_component_overlap: true,
        }
    }
    pub fn allow_overlap(&self) -> bool {
        self.allow_component_overlap
    }
    pub fn scope_type(&self) -> String {
        String::from("core.hydra.io/v1alpha1.Health")
    }
}
