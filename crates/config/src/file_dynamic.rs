//! Config file which server can edit at runtime.

use std::path::Path;

use error_stack::{Result, ResultExt};
use model::{BackendConfig, BotConfig};
use serde::{Deserialize, Serialize};
use simple_backend_config::file::ConfigFileUtils;
use toml_edit::{DocumentMut, Item};

use crate::file::ConfigFileError;

pub const CONFIG_FILE_DYNAMIC_NAME: &str = "server_config_dynamic.toml";

pub const DEFAULT_CONFIG_FILE_DYNAMIC_TEXT: &str = r#"

# Server can edit this config file at runtime.

# Enable automatic bots when server starts.
# Server can edit this table only if it is uncommented.
# [bots]
# users = 5
# admin = false

# Enable remote bot login API route.
# Server can edit this value only if it is uncommented.
# remote_bot_login = true

"#;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFileDynamic {
    #[serde(flatten)]
    pub backend_config: BackendConfig,
}

impl ConfigFileDynamic {
    pub fn minimal_config_for_api_doc_json() -> Self {
        Self {
            backend_config: BackendConfig {
                bots: None,
                remote_bot_login: None,
            },
        }
    }

    pub fn load(dir: impl AsRef<Path>) -> Result<ConfigFileDynamic, ConfigFileError> {
        let config_string = ConfigFileUtils::load_string(
            dir,
            CONFIG_FILE_DYNAMIC_NAME,
            DEFAULT_CONFIG_FILE_DYNAMIC_TEXT,
        )
        .change_context(ConfigFileError::SimpleBackendError)?;
        toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)
    }

    pub fn load_from_current_dir() -> Result<ConfigFileDynamic, ConfigFileError> {
        let current_dir = std::env::current_dir().change_context(ConfigFileError::LoadConfig)?;
        Self::load(current_dir)
    }

    pub fn edit_config_from_current_dir(
        bot_config: Option<BotConfig>,
        remote_bot_login: Option<bool>,
    ) -> Result<(), ConfigFileError> {
        let dir = std::env::current_dir().change_context(ConfigFileError::LoadConfig)?;

        let config = ConfigFileUtils::load_string(
            &dir,
            CONFIG_FILE_DYNAMIC_NAME,
            DEFAULT_CONFIG_FILE_DYNAMIC_TEXT,
        )
        .change_context(ConfigFileError::SimpleBackendError)?;

        let mut config_document = config
            .parse::<DocumentMut>()
            .change_context(ConfigFileError::EditConfig)?;

        if let Some(v) = bot_config {
            edit_document_bot_config(&mut config_document, v)?;
        }
        if let Some(v) = remote_bot_login {
            edit_document_remote_bot_login(&mut config_document, v)?;
        }

        let new_config = config_document.to_string();
        let file_path = dir.join(CONFIG_FILE_DYNAMIC_NAME);
        ConfigFileUtils::save_string(&file_path, &new_config)
            .change_context(ConfigFileError::SaveEditedConfig)
            .attach_printable(file_path.display().to_string())
    }
}

/// Edit BotConfig. Note that if the config ("bots" table) does not already
/// exist in the document, the document will not be edited.
fn edit_document_bot_config(
    config_document: &mut DocumentMut,
    bot_config: BotConfig,
) -> Result<(), ConfigFileError> {
    if let Some(Item::Table(bot_config_table)) = config_document.get_mut("bots") {
        if let Some(Item::Value(value)) = bot_config_table.get_mut("users") {
            *value = (bot_config.users as i64).into();
        } else {
            return Err(ConfigFileError::EditConfig)
                .attach_printable("The 'users' number field is missing from 'bots' table");
        }

        if let Some(Item::Value(value)) = bot_config_table.get_mut("admin") {
            *value = bot_config.admin.into();
        } else {
            return Err(ConfigFileError::EditConfig)
                .attach_printable("The 'admin' boolean field is missing from 'bots' table");
        }

        Ok(())
    } else {
        Err(ConfigFileError::EditConfig)
            .attach_printable("The config file does not have a 'bots' table")
    }
}

/// Edit `remote_bot_login` field. Note that if the field does not already
/// exist in the document, the document will not be edited.
fn edit_document_remote_bot_login(
    config_document: &mut DocumentMut,
    remote_bot_login: bool,
) -> Result<(), ConfigFileError> {
    if let Some(Item::Value(value)) = config_document.get_mut("remote_bot_login") {
        *value = remote_bot_login.into();
        Ok(())
    } else {
        Err(ConfigFileError::EditConfig)
            .attach_printable("The config file does not have a 'remote_bot_login' field")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    pub fn editing_bots_section_works() {
        let initial_document = r#"
            test = 1
            [test2]
            test3 = 1
            [bots]
            users = 0
            admin = false
        "#;
        let mut document = toml_edit::DocumentMut::from_str(initial_document).unwrap();

        let new_config = model::BotConfig {
            users: 1,
            admin: true,
        };

        edit_document_bot_config(&mut document, new_config).unwrap();

        let edited_document = document.to_string();
        let expected = r#"
            test = 1
            [test2]
            test3 = 1
            [bots]
            users = 1
            admin = true
        "#;

        assert_eq!(expected, edited_document);
    }

    #[test]
    pub fn editing_remote_bot_login_field_works() {
        let initial_document = r#"
            remote_bot_login = false
            test = 1
            [test2]
            test3 = 1
        "#;
        let mut document = toml_edit::DocumentMut::from_str(initial_document).unwrap();

        edit_document_remote_bot_login(&mut document, true).unwrap();

        let edited_document = document.to_string();
        let expected = r#"
            remote_bot_login = true
            test = 1
            [test2]
            test3 = 1
        "#;

        assert_eq!(expected, edited_document);
    }
}
