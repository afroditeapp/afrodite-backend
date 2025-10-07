use model::{AccountIdInternal, PendingNotificationFlags, PushNotificationStateInfoWithFlags};
use model_account::{EmailMessages, EmailSendingState};
use server_api::{
    app::{EmailSenderImpl, EventManagerProvider, GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_common::{data::DataError, result::Result};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_data_chat::write::GetWriteCommandsChat;
use server_data_profile::write::GetWriteCommandsProfile;
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
        Self::handle_custom_report_file_changes(&self.state).await?;
        Self::handle_client_features_file_changes(&self.state).await?;
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

    async fn handle_custom_report_file_changes(state: &S) -> Result<(), DataError> {
        let hash = if let Some(hash) = state.config().custom_reports_sha256() {
            hash.to_string()
        } else {
            return Ok(());
        };

        db_write_raw!(state, move |cmds| {
            cmds.account()
                .report()
                .update_custom_reports_sha256_and_sync_versions(hash)
                .await
        })
        .await
    }

    async fn handle_client_features_file_changes(state: &S) -> Result<(), DataError> {
        let hash = if let Some(hash) = state.config().client_features_sha256() {
            hash.to_string()
        } else {
            return Ok(());
        };

        db_write_raw!(state, move |cmds| {
            cmds.account()
                .client_featues()
                .update_client_features_sha256_and_sync_versions(hash)
                .await
        })
        .await
    }

    async fn handle_account_specific_tasks(
        state: &S,
        email_sender: EmailSenderImpl,
    ) -> Result<(), DataError> {
        let ids = state.read().common().account_ids_internal_vec().await?;

        for id in ids {
            // Email
            let email_state = state.read().account().email().email_state(id).await?;
            let send_if_needed = |state: &EmailSendingState, message: EmailMessages| {
                if *state == EmailSendingState::SendRequested {
                    email_sender.send(id, message)
                }
            };
            send_if_needed(
                &email_state.account_registered_state_number,
                EmailMessages::AccountRegistered,
            );

            db_write_raw!(state, move |cmds| {
                // Remove tmp files
                cmds.common().remove_tmp_files(id).await?;

                cmds.chat()
                    .limits()
                    .reset_daily_likes_left_if_needed(id)
                    .await?;

                // Automatic profile search notification state is stored only
                // in RAM, so it is not available anymore.
                cmds.events()
                    .remove_specific_pending_notification_flags_from_cache(
                        id,
                        PendingNotificationFlags::AUTOMATIC_PROFILE_SEARCH_COMPLETED,
                    )
                    .await;

                Ok(())
            })
            .await?;

            Self::send_push_notification_if_needed(state, id).await?;
        }

        Ok(())
    }

    async fn send_push_notification_if_needed(
        state: &S,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let push_notification_state = state
            .read()
            .common()
            .push_notification()
            .push_notification_state(id)
            .await?;

        match push_notification_state {
            PushNotificationStateInfoWithFlags::EmptyFlags => (),
            PushNotificationStateInfoWithFlags::WithFlags { info, .. } => {
                if info.push_notification_device_token.is_some() {
                    state.event_manager().trigger_push_notification_sending(id)
                }
            }
        }

        Ok(())
    }
}
