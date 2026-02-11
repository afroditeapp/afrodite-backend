use error_stack::ResultExt;
use futures::stream::{self, StreamExt};
use model::{AccountIdInternal, PushNotificationFlags, PushNotificationStateInfoWithFlags};
use model_account::{EmailMessages, EmailSendingState};
use serde_json;
use server_api::{
    app::{
        EmailSenderImpl, EventManagerProvider, GetConfig, GetProfileAttributes, ReadData, WriteData,
    },
    db_write_raw,
};
use server_common::{data::DataError, result::Result};
use server_data::{IntoDataError, read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use server_state::S;
use sha2::{Digest, Sha256};

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
        Self::handle_profile_attribute_schema_changes(&self.state).await?;
        Self::handle_custom_report_file_changes(&self.state).await?;
        Self::handle_client_features_file_changes(&self.state).await?;
        Self::handle_vapid_public_key_changes(&self.state).await?;
        Self::handle_account_specific_tasks(&self.state, &email_sender).await
    }

    async fn handle_profile_attribute_schema_changes(state: &S) -> Result<(), DataError> {
        let export = state.profile_attributes_manager().export();
        let json = serde_json::to_string(&export).change_context(DataError::Diesel)?;
        let hash = Sha256::digest(json.as_bytes());
        let hash_str = format!("{:x}", hash);

        let current_hash = state
            .read()
            .common()
            .profile_attributes()
            .profile_attributes_hash()
            .await?;

        let hash_changed = match current_hash {
            Some(h) => h != hash_str,
            None => true,
        };

        if hash_changed {
            db_write_raw!(state, move |cmds| {
                cmds.common()
                    .profile_attributes()
                    .upsert_profile_attributes_hash(&hash_str)
                    .await?;
                cmds.common()
                    .client_config()
                    .increment_client_config_sync_version_for_every_account()
                    .await
            })
            .await?;
        }

        Ok(())
    }

    async fn handle_custom_report_file_changes(state: &S) -> Result<(), DataError> {
        let hash = state.config().custom_reports_sha256().to_string();

        db_write_raw!(state, move |cmds| {
            cmds.account()
                .report()
                .update_custom_reports_sha256_and_sync_versions(hash)
                .await
        })
        .await
    }

    async fn handle_client_features_file_changes(state: &S) -> Result<(), DataError> {
        let hash = state.config().client_features_sha256().to_string();

        db_write_raw!(state, move |cmds| {
            cmds.account()
                .client_featues()
                .update_client_features_sha256_and_sync_versions(hash)
                .await
        })
        .await
    }

    async fn handle_vapid_public_key_changes(state: &S) -> Result<(), DataError> {
        let hash = state
            .config()
            .simple_backend()
            .web_push_config()
            .map(|(_, key)| Sha256::digest(key.get_public_key()))
            .map(|hash| format!("{hash:x}"))
            .unwrap_or_default();

        db_write_raw!(state, move |cmds| {
            cmds.common()
                .push_notification()
                .update_vapid_public_key_sha256_and_sync_versions(hash)
                .await
        })
        .await
    }

    async fn handle_account_specific_tasks(
        state: &S,
        email_sender: &EmailSenderImpl,
    ) -> Result<(), DataError> {
        let ids = state.read().common().account_ids_internal_vec().await?;

        let mut stream = stream::iter(ids)
            .map(|id| Self::handle_account(state, email_sender, id))
            .buffer_unordered(num_cpus::get());

        loop {
            match stream.next().await {
                Some(Ok(())) => (),
                Some(Err(e)) => return Err(e),
                None => return Ok(()),
            }
        }
    }

    async fn handle_account(
        state: &S,
        email_sender: &EmailSenderImpl,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        // Email
        let email_state = state.read().account().email().email_state(id).await?;
        for m in EmailMessages::VARIANTS {
            if *email_state.get_ref_to(*m) == EmailSendingState::SendRequested {
                email_sender.send(id, *m)
            }
        }

        state
            .read()
            .file_dir_write_access()
            .tmp_dir(id.into())
            .overwrite_and_remove_contents_if_exists()
            .await
            .into_data_error(id)?;

        if let Some(value) = state
            .read()
            .chat()
            .limits()
            .is_daily_likes_left_reset_needed(id)
            .await?
        {
            db_write_raw!(state, move |cmds| {
                cmds.chat()
                    .limits()
                    .reset_daily_likes_left(id, value.new_value)
                    .await?;
                Ok(())
            })
            .await?;
        }

        // Automatic profile search notification state is stored only
        // in RAM, so it is not available anymore.
        state
            .event_manager()
            .remove_pending_push_notification_flags_from_cache(
                id,
                PushNotificationFlags::AUTOMATIC_PROFILE_SEARCH_COMPLETED,
            )
            .await;

        Self::send_push_notification_if_needed(state, id).await?;

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
