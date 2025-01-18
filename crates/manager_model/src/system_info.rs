
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct SystemInfoList {
    pub info: Vec<SystemInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct SystemInfo {
    pub name: String,
    pub info: Vec<CommandOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct CommandOutput {
    pub name: String,
    pub output: String,
}
