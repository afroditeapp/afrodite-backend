use error_stack::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_database::data::create_dirs_and_get_simple_backend_dir_path;
use tokio::io::AsyncWriteExt;

use super::EmailError;

const EMAIL_SENDER_STATE_FILE: &str = "email_sender_state.toml";

// TODO(prod): Save counter reset time to EmailLimitStateStorage.

/// Save emal sender limit states before closing the server.
///
/// It is assumed that server restarts quickly enough so time related
/// limit invalidation is not implemented.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EmailLimitStateStorage {
    pub emails_sent_per_minute: u32,
    pub emails_sent_per_day: u32,
}

impl EmailLimitStateStorage {
    pub async fn load_and_remove(config: &SimpleBackendConfig) -> Result<Self, EmailError> {
        let config_dir = create_dirs_and_get_simple_backend_dir_path(config)
            .change_context(EmailError::LoadSavedStateFailed)?;

        let config_file = config_dir.join(EMAIL_SENDER_STATE_FILE);
        if config_file.exists() {
            let loaded_file = tokio::fs::read_to_string(&config_file)
                .await
                .change_context(EmailError::LoadSavedStateFailed)?;
            let loaded_config: EmailLimitStateStorage =
                toml::from_str(&loaded_file).change_context(EmailError::LoadSavedStateFailed)?;

            tokio::fs::remove_file(config_file)
                .await
                .change_context(EmailError::RemovingSavedStateFailed)?;

            Ok(loaded_config)
        } else {
            Ok(Self::default())
        }
    }

    pub async fn save(&self, config: &SimpleBackendConfig) -> Result<(), EmailError> {
        let config_dir = create_dirs_and_get_simple_backend_dir_path(config)
            .change_context(EmailError::SavingStateFailed)?;

        let config_file = config_dir.join(EMAIL_SENDER_STATE_FILE);

        let config_string =
            toml::to_string_pretty(self).change_context(EmailError::SavingStateFailed)?;
        let mut file = tokio::fs::File::create(config_file)
            .await
            .change_context(EmailError::SavingStateFailed)?;
        file.write_all(config_string.as_bytes())
            .await
            .change_context(EmailError::SavingStateFailed)?;

        Ok(())
    }
}
