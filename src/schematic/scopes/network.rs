/// Network
#[derive(Clone, Debug)]
pub struct Network {
    pub name: String,
    pub allow_component_overlap: bool,
}

impl Network {
    pub fn from_params(name: String) -> Self {
        // Right now, we're relying on the higher level validation logic to validate types.
        Network {
            name: name,
            allow_component_overlap: true,
        }
    }
    pub fn allow_overlap(&self) -> bool {
        self.allow_component_overlap
    }
}
