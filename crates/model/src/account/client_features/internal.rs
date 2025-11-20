use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{
    ChatConfig, FeaturesConfig, LikesConfig, MapConfig, NewsConfig, ProfileConfig, ServerConfig,
};
use crate::{ClientFeaturesConfig, ScheduledTasksConfig};

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
