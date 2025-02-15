use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod report;
pub use report::*;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct BackendConfig {
    pub bots: Option<BotConfig>,
    pub remote_bot_login: Option<bool>,
}

impl BackendConfig {
    pub fn empty() -> Self {
        BackendConfig::default()
    }
}

/// Enable automatic bots when server starts.
/// Editing of this field with edit module is only allowed when
/// this exists in the config file.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct BotConfig {
    /// User bot count
    pub users: u32,
    /// Admin bot
    pub admin: bool,
}
