use server_api::{
    app::{ReadData, WriteData},
    db_write_raw,
};
use server_common::{data::DataError, result::Result};
use server_data::read::GetReadCommandsCommon;
use server_data_profile::write::GetWriteCommandsProfile;
use server_state::S;

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
        Self::handle_account_specific_tasks(&self.state).await
    }

    async fn handle_account_specific_tasks(state: &S) -> Result<(), DataError> {
        let ids = state.read().common().account_ids_internal_vec().await?;

        for id in ids {
            db_write_raw!(state, move |cmds| {
                cmds.profile()
                    .update_last_seen_time_from_cache_to_database(id)
                    .await
            })
            .await?;
        }

        Ok(())
    }
}
