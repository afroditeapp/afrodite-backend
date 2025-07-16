use server_api::{
    app::{GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_common::{data::DataError, result::Result};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
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
    /// - [simple_backend::email::EmailManager::before_quit]
    /// - [server_common::push_notifications::PushNotificationManager::quit_logic]
    pub async fn run_and_wait_completion(self) -> Result<(), DataError> {
        Self::handle_account_specific_tasks(&self.state).await?;
        if self.state.config().components().account {
            TaskUtils::save_client_version_statistics(&self.state).await?;
        }
        TaskUtils::save_api_usage_statistics(&self.state).await?;
        TaskUtils::save_ip_address_statistics(&self.state).await
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
