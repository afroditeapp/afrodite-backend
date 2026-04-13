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

mod admin_bot;
pub use admin_bot::*;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct BotConfig {
    /// Enable remote bot login API
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub remote_bot_login: bool,
    /// Admin bot enabled
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub admin_bot: bool,
    /// User bot count
    #[serde(default, skip_serializing_if = "is_zero")]
    #[schema(default = 0)]
    pub user_bots: u32,
    /// Admin bot config
    pub admin_bot_config: AdminBotConfig,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct BotConfigWarnings {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error: bool,
    /// True, when getting warnings fails because admin bot is offline
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_admin_bot_offline: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_name_moderation_file_config_missing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profile_text_moderation_file_config_missing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub content_moderation_file_config_missing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub face_verification_file_config_missing: bool,
}

impl BotConfigWarnings {
    pub fn error_admin_bot_offline() -> Self {
        Self {
            error: true,
            error_admin_bot_offline: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct DynamicServerConfig {}
