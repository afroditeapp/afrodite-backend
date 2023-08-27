//! Config file editing

use std::path::Path;

use error_stack::{Result, ResultExt};
use toml_edit::{Document, Item, Value};

use crate::file::{BotConfig, ConfigFile, ConfigFileError};

pub fn edit_bot_config(
    dir: impl AsRef<Path>,
    bot_config: BotConfig,
) -> Result<(), ConfigFileError> {
    let config = ConfigFile::load_as_string(&dir)?;
    let mut config_document = config
        .parse::<Document>()
        .change_context(ConfigFileError::EditConfig)?;

    edit_document_bot_config(&mut config_document, bot_config)?;

    let new_config = config_document.to_string();
    ConfigFile::save_string_as_config(dir, &new_config)
        .change_context(ConfigFileError::SaveEditedConfig)
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

    use super::edit_bot_config;
    use crate::{edit::edit_document_bot_config, file::BotConfig};

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

        let new_config = BotConfig {
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
