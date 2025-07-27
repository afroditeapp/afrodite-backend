use std::path::Path;

use error_stack::{Result, ResultExt};
use model::StringResource;
use serde::Deserialize;

use crate::file::ConfigFileError;

pub enum NotificationStringResource {
    NewNotificationAvailableTitle,
}

#[derive(Debug, Default, Deserialize)]
pub struct NotificationContentFile {
    pub new_notification_available_title: Option<StringResource>,
    pub new_notification_available_body: Option<StringResource>,
    #[serde(flatten)]
    pub other: toml::Table,
}

impl NotificationContentFile {
    pub fn load(file: impl AsRef<Path>) -> Result<NotificationContentFile, ConfigFileError> {
        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let config: NotificationContentFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        if let Some(key) = config.other.keys().next() {
            return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                "Notification content config file error. Unknown string resource '{key}'."
            ));
        }

        Ok(config)
    }

    pub fn get_value(&self, resource: NotificationStringResource, language: &str) -> String {
        let (translations, default) = match resource {
            NotificationStringResource::NewNotificationAvailableTitle => (
                &self.new_notification_available_title,
                "New notification available",
            ),
        };

        translations
            .as_ref()
            .map(|v| v.translations.get(language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or(default.to_string())
    }
}
