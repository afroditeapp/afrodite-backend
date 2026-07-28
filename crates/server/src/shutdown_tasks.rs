use server_api::{
    app::{GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_common::{data::DataError, result::Result};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_data_account::write::GetWriteCommandsAccount;
use server_data_profile::write::GetWriteCommandsProfile;
use server_state::S;

use crate::task_utils::TaskUtils;

pub struct ShutdownTasks {
    state: S,
}

impl ShutdownTasks {
    pub fn new(state: S) -> Self {
        Self { state }
    }

    /// Other quit tasks not located here:
    /// - [simple_backend::email::SmtpClient::save_state]
    /// - [server_common::push_notifications::PushNotificationManager::quit_logic]
    pub async fn run_and_wait_completion(self) -> Result<(), DataError> {
        Self::persist_email_login_tokens(&self.state).await?;
        Self::handle_account_specific_tasks(&self.state).await?;
        TaskUtils::save_client_version_statistics(&self.state).await?;
        TaskUtils::save_api_usage_statistics(&self.state).await?;
        TaskUtils::save_ip_address_statistics(&self.state).await
    }

    /// Persist in-memory email login tokens to DB so they survive a restart.
    async fn persist_email_login_tokens(state: &S) -> Result<(), DataError> {
        let validity = state
            .config()
            .limits_account()
            .email_login_token_validity_duration;

        let tokens = state
            .email_registration_tokens()
            .drain_valid_login_tokens(validity)
            .await;

        if tokens.is_empty() {
            return Ok(());
        }

        db_write_raw!(state, move |cmds| {
            cmds.account()
                .email()
                .replace_all_email_login_tokens(tokens)
                .await
        })
        .await
    }

    async fn handle_account_specific_tasks(state: &S) -> Result<(), DataError> {
        let ids = state.read().common().account_ids_internal_vec().await?;

        for id in ids {
            db_write_raw!(state, move |cmds| {
                cmds.common()
                    .save_authentication_tokens_from_cache_to_db_if_needed(id)
                    .await?;
                cmds.profile()
                    .update_last_seen_time_from_cache_to_database(id)
                    .await
            })
            .await?;
        }

        Ok(())
    }
}
