use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Profile {
    name: String,
}

impl Profile {
    pub fn new(name: String) -> Self {
        Self {name}
    }
}
