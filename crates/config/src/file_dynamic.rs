//! Config file which server can edit at runtime.

use std::path::Path;

use error_stack::{Result, ResultExt};
use model::BackendConfig;
use serde::{Deserialize, Serialize};
use simple_backend_config::file::ConfigFileUtils;
use toml_edit::{DocumentMut, Item};

use crate::file::ConfigFileError;

pub const CONFIG_FILE_DYNAMIC_NAME: &str = "server_config_dynamic.toml";

pub const DEFAULT_CONFIG_FILE_DYNAMIC_TEXT: &str = r#"

# Server can edit this config file at runtime.
# Only fields which are uncommented can be edited.

# remote_bot_login = true

# [local_bots]
# users = 5
# admin = false

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
                local_bots: None,
                remote_bot_login: None,
            },
        }
    }

    pub fn load(
        dir: impl AsRef<Path>,
        save_default_if_not_found: bool,
    ) -> Result<ConfigFileDynamic, ConfigFileError> {
        let config_string = ConfigFileUtils::load_string(
            dir,
            CONFIG_FILE_DYNAMIC_NAME,
            DEFAULT_CONFIG_FILE_DYNAMIC_TEXT,
            save_default_if_not_found,
        )
        .change_context(ConfigFileError::SimpleBackendError)?;
        toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)
    }

    pub fn load_from_current_dir(
        save_default_if_not_found: bool,
    ) -> Result<ConfigFileDynamic, ConfigFileError> {
        let current_dir = std::env::current_dir().change_context(ConfigFileError::LoadConfig)?;
        Self::load(current_dir, save_default_if_not_found)
    }

    pub fn edit_config_from_current_dir(
        remote_bot_login: Option<bool>,
        admin_bot: Option<bool>,
        user_bots: Option<u32>,
    ) -> Result<(), ConfigFileError> {
        let dir = std::env::current_dir().change_context(ConfigFileError::LoadConfig)?;

        let config = ConfigFileUtils::load_string(
            &dir,
            CONFIG_FILE_DYNAMIC_NAME,
            DEFAULT_CONFIG_FILE_DYNAMIC_TEXT,
            true,
        )
        .change_context(ConfigFileError::SimpleBackendError)?;

        let mut config_document = config
            .parse::<DocumentMut>()
            .change_context(ConfigFileError::EditConfig)?;

        if let Some(v) = remote_bot_login {
            edit_document_remote_bot_login_field_if_it_exists(&mut config_document, v)?;
        }

        if let Some(v) = admin_bot {
            edit_document_admin_field_if_it_exists(&mut config_document, v)?;
        }

        if let Some(v) = user_bots {
            edit_document_users_field_if_it_exists(&mut config_document, v)?;
        }

        let new_config = config_document.to_string();
        let file_path = dir.join(CONFIG_FILE_DYNAMIC_NAME);
        ConfigFileUtils::save_string(&file_path, &new_config)
            .change_context(ConfigFileError::SaveEditedConfig)
            .attach_printable(file_path.display().to_string())
    }
}

fn edit_document_remote_bot_login_field_if_it_exists(
    config_document: &mut DocumentMut,
    remote_bot_login: bool,
) -> Result<(), ConfigFileError> {
    if let Some(Item::Value(value)) = config_document.get_mut("remote_bot_login") {
        *value = remote_bot_login.into();
        Ok(())
    } else {
        Err(ConfigFileError::EditConfig)
            .attach_printable("Editing dynamic config field 'remote_bot_login' is disabled")
    }
}

fn edit_document_admin_field_if_it_exists(
    config_document: &mut DocumentMut,
    admin_bot: bool,
) -> Result<(), ConfigFileError> {
    let field = config_document
        .get_mut("local_bots")
        .and_then(|v| v.as_table_mut())
        .and_then(|v| v.get_mut("admin"));
    if let Some(Item::Value(value)) = field {
        *value = admin_bot.into();
        Ok(())
    } else {
        Err(ConfigFileError::EditConfig)
            .attach_printable("Editing dynamic config field 'admin' is disabled")
    }
}

fn edit_document_users_field_if_it_exists(
    config_document: &mut DocumentMut,
    users: u32,
) -> Result<(), ConfigFileError> {
    let field = config_document
        .get_mut("local_bots")
        .and_then(|v| v.as_table_mut())
        .and_then(|v| v.get_mut("users"));
    if let Some(Item::Value(value)) = field {
        *value = Into::<i64>::into(users).into();
        Ok(())
    } else {
        Err(ConfigFileError::EditConfig)
            .attach_printable("Editing dynamic config field 'users' is disabled")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    pub fn editing_remote_bot_login_field_works() {
        let initial_document = r#"
            remote_bot_login = false
            test = 1
            [test2]
            test3 = 1
        "#;
        let mut document = toml_edit::DocumentMut::from_str(initial_document).unwrap();

        edit_document_remote_bot_login_field_if_it_exists(&mut document, true).unwrap();

        let edited_document = document.to_string();
        let expected = r#"
            remote_bot_login = true
            test = 1
            [test2]
            test3 = 1
        "#;

        assert_eq!(expected, edited_document);
    }

    #[test]
    pub fn editing_admin_field_works() {
        let initial_document = r#"
            test = 1
            [test2]
            test3 = 1
            [local_bots]
            admin = false
        "#;
        let mut document = toml_edit::DocumentMut::from_str(initial_document).unwrap();

        edit_document_admin_field_if_it_exists(&mut document, true).unwrap();

        let edited_document = document.to_string();
        let expected = r#"
            test = 1
            [test2]
            test3 = 1
            [local_bots]
            admin = true
        "#;

        assert_eq!(expected, edited_document);
    }

    #[test]
    pub fn editing_users_field_works() {
        let initial_document = r#"
            test = 1
            [test2]
            test3 = 1
            [local_bots]
            users = 0
        "#;
        let mut document = toml_edit::DocumentMut::from_str(initial_document).unwrap();

        edit_document_users_field_if_it_exists(&mut document, 1).unwrap();

        let edited_document = document.to_string();
        let expected = r#"
            test = 1
            [test2]
            test3 = 1
            [local_bots]
            users = 1
        "#;

        assert_eq!(expected, edited_document);
    }
}
