use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct BackendConfig {
    pub bots: Option<BotConfig>,
}

/// Enable automatic bots when server starts.
/// Editing of this field with edit module is only allowed when
/// this exists in the config file.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct BotConfig {
    /// User bot count
    pub users: u32,
    /// Admin bot count
    pub admins: u32,
}
