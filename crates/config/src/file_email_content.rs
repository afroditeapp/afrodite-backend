use std::path::Path;

use error_stack::{Result, ResultExt};
use model::EmailMessages;
use serde::Deserialize;

use crate::file::ConfigFileError;

#[derive(Debug, Default, Deserialize)]
pub struct EmailContentFile {
    #[serde(default)]
    pub email: Vec<EmailContent>,
}

impl EmailContentFile {
    pub fn load(file: impl AsRef<Path>) -> Result<EmailContentFile, ConfigFileError> {
        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let config: EmailContentFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        let mut messages = std::collections::HashSet::<EmailMessages>::new();
        for content in &config.email {
            if messages.contains(&content.message_type) {
                return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                    "Message {:?} is defined more than once",
                    content.message_type
                ));
            }
            messages.insert(content.message_type);
        }

        let mut missing_messages = std::collections::HashSet::<EmailMessages>::new();
        for msg_type in EmailMessages::VARIANTS {
            if !messages.contains(msg_type) {
                missing_messages.insert(*msg_type);
            }
        }

        if !missing_messages.is_empty() {
            return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                "Message content not defined for {:?}",
                missing_messages
            ));
        }

        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
pub struct EmailContent {
    pub message_type: EmailMessages,
    pub subject: String,
    pub body: String,
}
