//! Config file which server can edit at runtime.

use std::path::Path;

use error_stack::{Result, ResultExt};
use model::{BotConfig, BackendConfig};
use serde::{Deserialize, Serialize};
use toml_edit::{Document, Item, Value};

use crate::{file::{ConfigFile, ConfigFileError, ConfigFileUtils}};

pub const CONFIG_FILE_DYNAMIC_NAME: &str = "server_config_dynamic.toml";


pub const DEFAULT_CONFIG_FILE_DYNAMIC_TEXT: &str = r#"

# Server can edit this config file at runtime.

# Enable automatic bots when server starts.
# Set also internal API setting bot_login to true to allow bots
# to connect to the server. Server can edit this table only if
# it is uncommented.
# [bots]
# users = 5
# admins = 1

"#;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFileDynamic {
    #[serde(flatten)]
    pub backend_config: BackendConfig,
}

impl ConfigFileDynamic {
    pub fn load(dir: impl AsRef<Path>) -> Result<ConfigFileDynamic, ConfigFileError> {
        let config_string = ConfigFileUtils::load_string(
            dir,
            CONFIG_FILE_DYNAMIC_NAME,
            DEFAULT_CONFIG_FILE_DYNAMIC_TEXT,
        )?;
        toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)
    }

    pub fn load_from_current_dir() -> Result<ConfigFileDynamic, ConfigFileError> {
        let current_dir = std::env::current_dir().change_context(ConfigFileError::LoadConfig)?;
        Self::load(current_dir)
    }

    pub fn edit_bot_config_from_current_dir(
        bot_config: BotConfig,
    ) -> Result<(), ConfigFileError> {
        let dir = std::env::current_dir().change_context(ConfigFileError::LoadConfig)?;

        let config = ConfigFileUtils::load_string(
            &dir,
            CONFIG_FILE_DYNAMIC_NAME,
            DEFAULT_CONFIG_FILE_DYNAMIC_TEXT,
        )?;

        let mut config_document = config
            .parse::<Document>()
            .change_context(ConfigFileError::EditConfig)?;

        edit_document_bot_config(&mut config_document, bot_config)?;

        let new_config = config_document.to_string();
        ConfigFileUtils::save_string(dir, &new_config)
            .change_context(ConfigFileError::SaveEditedConfig)
    }
}

/// Edit BotConfig. Note that if the config ("bots" table) does not already
/// exist in the document, the document will not be edited.
fn edit_document_bot_config(
    config_document: &mut Document,
    bot_config: BotConfig,
) -> Result<(), ConfigFileError> {
    if let Some(Item::Table(bot_config_table)) = config_document.get_mut("bots") {
        if let Some(Item::Value(value)) = bot_config_table.get_mut("users") {
            *value = (bot_config.users as i64).into();
        } else {
            return Err(ConfigFileError::EditConfig)
                .attach_printable("The 'users' number field is missing from 'bots' table");
        }

        if let Some(Item::Value(value)) = bot_config_table.get_mut("admins") {
            *value = (bot_config.admins as i64).into();
        } else {
            return Err(ConfigFileError::EditConfig)
                .attach_printable("The 'admins' number field is missing from 'bots' table");
        }

        Ok(())
    } else {
        Err(ConfigFileError::EditConfig)
            .attach_printable("The config file does not have a 'bots' table")
    }
}

#[cfg(test)]
mod tests {
    use std::{str::FromStr, string};

    use super::*;

    #[test]
    pub fn editing_bots_section_works() {
        let toml_with_no_bots_section = r#"
            test = 1
            [test2]
            test3 = 1
            [bots]
            users = 0
            admins = 0
        "#;
        let mut document = toml_edit::Document::from_str(toml_with_no_bots_section).unwrap();

        let new_config = model::BotConfig {
            users: 1,
            admins: 1,
        };

        edit_document_bot_config(&mut document, new_config).unwrap();

        let edited_document = document.to_string();
        let expected = r#"
            test = 1
            [test2]
            test3 = 1
            [bots]
            users = 1
            admins = 1
        "#;

        assert_eq!(expected, edited_document);
    }
}
