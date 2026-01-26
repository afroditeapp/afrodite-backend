use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{
    ChatConfig, FeaturesConfig, LikesConfig, MapConfig, NewsConfig, ProfileConfig, ServerConfig,
};
use crate::{ClientFeaturesConfig, ScheduledTasksConfig};

const DEFAULT_CONFIG_FILE_TEXT: &str = r#"
[attribution.generic]
default = "Generic data attribution"

[attribution.ip_country]
default = "IP address country data attribution"

[features]
video_calls = true

[news]
locales = []

[limits.likes]
unlimited_likes_disabling_time = "2:00"

[limits.likes.daily]
daily_likes = 5
reset_time = "2:00"

[map.bounds]
top_left = { lat = 90, lon = -180 }
bottom_right = { lat = -90, lon = 180 }

[map.zoom]
min = 1
max = 15
max_tile_downloading = 13
location_not_selected = 6
location_selected = 10

[map.initial_location]
lat = 0
lon = 0

[profile]
# On iOS with default keyboard settings, ' is ‘ or ’.
profile_name_regex = "^[-'‘’.A-Za-z]+$"

[chat]
message_state_seen = true

[chat.typing_indicator]
min_wait_seconds_between_requests_server = 1
min_wait_seconds_between_requests_client = 4
start_event_ttl_seconds = 10

[chat.check_online_status]
min_wait_seconds_between_requests_server = 4
min_wait_seconds_between_requests_client = 8

[server.scheduled_tasks]
daily_start_time = "3:00"

"#;

/// Client features config file
#[derive(Debug, Default, Clone, Deserialize)]
pub struct ClientFeaturesConfigInternal {
    #[serde(default)]
    pub attribution: AttributionConfigInternal,
    #[serde(default)]
    pub features: FeaturesConfig,
    pub news: Option<NewsConfig>,
    #[serde(default)]
    pub map: MapConfig,
    #[serde(default)]
    pub likes: LikesConfig,
    #[serde(default)]
    pub profile: ProfileConfig,
    #[serde(default)]
    pub chat: ChatConfig,
    #[serde(default)]
    pub server: ServerConfig,
}

impl ClientFeaturesConfigInternal {
    pub const CONFIG_FILE_NAME: &str = "client_features.toml";
    pub const DEFAULT_CONFIG_FILE_TEXT: &str = DEFAULT_CONFIG_FILE_TEXT;

    pub fn to_client_features_config(self) -> Result<ClientFeaturesConfig, String> {
        if let Some(key) = self.attribution.other.keys().next() {
            return Err(format!(
                "Client features config file error. Unknown attribution string resource '{key}'."
            ));
        }

        if let Some(regex) = &self.profile.profile_name_regex {
            if !regex.starts_with('^') {
                return Err("Profile name regex does not start with '^'".to_string());
            }
            if !regex.ends_with('$') {
                return Err("Profile name regex does not end with '$'".to_string());
            }
            Regex::new(regex).map_err(|v| v.to_string())?;
        }

        Ok(ClientFeaturesConfig {
            attribution: Some(self.attribution.into()),
            features: self.features.into(),
            news: self.news,
            map: self.map.into(),
            likes: self.likes.into(),
            profile: self.profile.into(),
            chat: self.chat.into(),
            server: self.server.into(),
        })
    }

    pub fn scheduled_tasks(&self) -> ScheduledTasksConfig {
        self.server.scheduled_tasks.clone().unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AttributionConfigInternal {
    pub generic: Option<StringResourceInternal>,
    pub ip_country: Option<StringResourceInternal>,
    #[serde(flatten)]
    pub other: toml::Table,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StringResourceInternal {
    pub default: String,
    /// Keys are country codes like "en".
    #[serde(flatten)]
    pub translations: HashMap<String, String>,
}

impl StringResourceInternal {
    pub fn values(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.default.as_str()).chain(self.translations.values().map(|v| v.as_str()))
    }

    pub fn all_strings_contain(&self, text: &str) -> bool {
        for v in self.values() {
            if !v.contains(text) {
                return false;
            }
        }

        true
    }
}
