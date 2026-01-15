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
    #[serde(default)]
    pub remote_bot_login: bool,
    /// Admin bot enabled
    #[serde(default)]
    pub admin_bot: bool,
    /// User bot count
    #[serde(default)]
    pub user_bots: u32,
}

impl BackendConfig {
    pub fn empty() -> Self {
        BackendConfig::default()
    }
}
