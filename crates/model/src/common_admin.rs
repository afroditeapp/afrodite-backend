use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

mod notification;
pub use notification::*;

mod report;
pub use report::*;

mod api_usage;
pub use api_usage::*;

mod ip_address;
pub use ip_address::*;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct BackendConfig {
    /// Enable remote bot login API
    ///
    /// If None, editing the value is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_bot_login: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_bots: Option<LocalBotsConfig>,
}

impl BackendConfig {
    pub fn empty() -> Self {
        BackendConfig::default()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct LocalBotsConfig {
    /// Admin bot
    ///
    /// If None, editing the value is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
    /// User bot count
    ///
    /// If None, editing the value is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<u32>,
}
