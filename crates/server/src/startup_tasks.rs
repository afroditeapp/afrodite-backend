use model_account::{EmailMessages, EmailSendingState};
use server_api::{app::{EmailSenderImpl, ReadData, WriteData}, db_write_raw};
use server_common::{app::GetConfig, data::DataError, result::Result};
use server_data::write::GetWriteCommandsCommon;
use server_data_profile::write::GetWriteCommandsProfile;
use server_data_account::read::GetReadCommandsAccount;
use server_state::S;

pub struct StartupTasks {
    state: S,
}

impl StartupTasks {
    pub fn new(state: S) -> Self {
        Self { state }
    }

    pub async fn run_and_wait_completion(
        self,
        email_sender: EmailSenderImpl,
    ) -> Result<(), DataError> {
        Self::handle_profile_attribute_file_changes(&self.state).await?;
        Self::handle_account_specific_tasks(&self.state, email_sender).await
    }

    async fn handle_profile_attribute_file_changes(state: &S) -> Result<(), DataError> {
        let hash = if let Some(hash) = state.config().profile_attributes_sha256() {
            hash.to_string()
        } else {
            return Ok(());
        };

        db_write_raw!(state, move |cmds| {
            cmds.profile()
                .update_profile_attributes_sha256_and_sync_versions(hash)
                .await
        })
        .await
    }

    async fn handle_account_specific_tasks(state: &S, email_sender: EmailSenderImpl) -> Result<(), DataError> {
        let ids = state.read().account().account_ids_internal_vec().await?;

        for id in ids {
            // Email
            let email_state = state.read().account().email().email_state(id).await?;
            let send_if_needed = |
                state: &EmailSendingState,
                message: EmailMessages
            | {
                if *state == EmailSendingState::SendRequested {
                    email_sender.send(id, message)
                }
            };
            send_if_needed(&email_state.account_registered_state_number, EmailMessages::AccountRegistered);

            db_write_raw!(state, move |cmds| {
                // FCM
                // The pending notification flags are already loaded from
                // database to cache.
                cmds.events()
                    .trigger_push_notification_sending_check_if_needed(id)
                    .await;

                // Remove tmp files
                cmds.common()
                    .remove_tmp_files(id).await?;

                Ok(())
            })
            .await?;
        }

        Ok(())
    }
}
