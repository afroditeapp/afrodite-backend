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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct BotConfigWarnings {
    pub profile_name_moderation_file_config_missing: bool,
    pub profile_text_moderation_file_config_missing: bool,
    pub content_moderation_file_config_missing: bool,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}
