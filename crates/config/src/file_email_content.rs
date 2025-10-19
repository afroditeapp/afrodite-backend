use std::{io::Write, path::Path};

use error_stack::{Result, ResultExt};
use model::StringResourceInternal;
use serde::Deserialize;

use crate::file::ConfigFileError;

const DEFAULT_EMAIL_CONTENT: &str = r#"

# Account registered

[account_registered_subject]
default = "New account created"

[account_registered_body]
default = "You created a new account"

# New message

[new_message_subject]
default = "New message received"

[new_message_body]
default = "You have received a new message"

# New like

[new_like_subject]
default = "New chat request received"

[new_like_body]
default = "You have received a new chat request"

"#;

#[derive(Debug, Default, Deserialize)]
pub struct EmailContentFile {
    pub account_registered_subject: Option<StringResourceInternal>,
    pub account_registered_body: Option<StringResourceInternal>,
    pub new_message_subject: Option<StringResourceInternal>,
    pub new_message_body: Option<StringResourceInternal>,
    pub new_like_subject: Option<StringResourceInternal>,
    pub new_like_body: Option<StringResourceInternal>,
    #[serde(flatten)]
    pub other: toml::Table,
}

impl EmailContentFile {
    pub fn load(
        file: impl AsRef<Path>,
        save_if_needed: bool,
    ) -> Result<EmailContentFile, ConfigFileError> {
        let path = file.as_ref();
        if !path.exists() && save_if_needed {
            let mut new_file =
                std::fs::File::create_new(path).change_context(ConfigFileError::LoadConfig)?;
            new_file
                .write_all(DEFAULT_EMAIL_CONTENT.as_bytes())
                .change_context(ConfigFileError::LoadConfig)?;
        }
        let config_content =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let config: EmailContentFile =
            toml::from_str(&config_content).change_context(ConfigFileError::LoadConfig)?;

        if let Some(key) = config.other.keys().next() {
            return Err(ConfigFileError::InvalidConfig).attach_printable(format!(
                "Email content config file error. Unknown string resource '{key}'."
            ));
        }

        Ok(config)
    }

    pub fn get<'a, T: AsRef<str>>(&'a self, language: Option<&'a T>) -> EmailStringGetter<'a> {
        EmailStringGetter {
            config: self,
            language: language.map(|v| v.as_ref()).unwrap_or_default(),
        }
    }
}

pub struct EmailStringGetter<'a> {
    config: &'a EmailContentFile,
    language: &'a str,
}

impl<'a> EmailStringGetter<'a> {
    pub fn account_registered_subject(&self) -> String {
        self.config
            .account_registered_subject
            .as_ref()
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or("New account created".to_string())
    }

    pub fn account_registered_body(&self) -> String {
        self.config
            .account_registered_body
            .as_ref()
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or("You created a new account".to_string())
    }

    pub fn new_message_subject(&self) -> String {
        self.config
            .new_message_subject
            .as_ref()
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or("New message received".to_string())
    }

    pub fn new_message_body(&self) -> String {
        self.config
            .new_message_body
            .as_ref()
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or("You have received a new message".to_string())
    }

    pub fn new_like_subject(&self) -> String {
        self.config
            .new_like_subject
            .as_ref()
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or("New chat request received".to_string())
    }

    pub fn new_like_body(&self) -> String {
        self.config
            .new_like_body
            .as_ref()
            .map(|v| v.translations.get(self.language).unwrap_or(&v.default))
            .cloned()
            .unwrap_or("You have received a new chat request".to_string())
    }
}
