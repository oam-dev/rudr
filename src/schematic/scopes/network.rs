/// Network
use crate::schematic::parameter::ParameterValue;
use crate::schematic::scopes::NETWORK_SCOPE;

#[derive(Clone, Debug)]
pub struct Network {
    pub name: String,
    pub allow_component_overlap: bool,
}

impl Network {
    pub fn from_params(name: String, _params: Option<Vec<ParameterValue>>) -> Self {
        // Right now, we're relying on the higher level validation logic to validate types.
        Network {
            name,
            allow_component_overlap: true,
        }
    }
    pub fn allow_overlap(&self) -> bool {
        self.allow_component_overlap
    }
    pub fn scope_type(&self) -> String {
        String::from(NETWORK_SCOPE)
    }
}
