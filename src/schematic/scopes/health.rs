use crate::schematic::parameter::ParameterValue;
use crate::schematic::scopes::HEALTH_SCOPE;

/// Health
#[derive(Clone, Debug)]
pub struct Health {
    pub name: String,
    pub allow_component_overlap: bool,
}

impl Health {
    pub fn from_params(name: String, _params: Option<Vec<ParameterValue>>) -> Self {
        // Right now, we're relying on the higher level validation logic to validate types.
        Health {
            name,
            allow_component_overlap: true,
        }
    }
    pub fn allow_overlap(&self) -> bool {
        self.allow_component_overlap
    }
    pub fn scope_type(&self) -> String {
        String::from(HEALTH_SCOPE)
    }
}
