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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AccountRegistrationPlatforms {
    #[serde(default = "default_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub android: bool,
    #[serde(default = "default_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub ios: bool,
    #[serde(default = "default_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub web: bool,
}

impl Default for AccountRegistrationPlatforms {
    fn default() -> Self {
        Self {
            android: true,
            ios: true,
            web: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AccountLoginPlatforms {
    #[serde(default = "default_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub android: bool,
    #[serde(default = "default_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub ios: bool,
    #[serde(default = "default_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub web: bool,
}

impl Default for AccountLoginPlatforms {
    fn default() -> Self {
        Self {
            android: true,
            ios: true,
            web: true,
        }
    }
}

fn default_true() -> bool {
    true
}

fn value_is_true(v: &bool) -> bool {
    *v
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct DynamicServerConfig {
    pub account_registration_platforms: AccountRegistrationPlatforms,
    pub account_login_platforms: AccountLoginPlatforms,
}
