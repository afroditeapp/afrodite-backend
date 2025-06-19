use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct SystemInfo {
    pub info: Vec<CommandOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct CommandOutput {
    pub name: String,
    pub output: String,
}
