use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct Profile {
    name: String,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Self {name}
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
