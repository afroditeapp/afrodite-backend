use std::net::SocketAddr;

use config::{file::ConfigFileError, file_dynamic::ConfigFileDynamic, Config};
use error_stack::ResultExt;
use futures::Future;
use model::{
    AccessToken, AccountId, AccountIdInternal, AccountState, BackendConfig, BackendVersion, EmailMessages, PendingNotificationFlags, Permissions, PublicKeyIdAndVersion, PushNotificationStateInfoWithFlags
};
use crate::{db_write_raw, internal_api::InternalApiClient};
use server_common::push_notifications::{PushNotificationError, PushNotificationStateProvider};
use server_data::{
    content_processing::ContentProcessingManagerData, db_manager::RouterDatabaseReadHandle, event::EventManagerWithCacheReference, read::GetReadCommandsCommon, write_commands::WriteCmds, write_concurrent::{ConcurrentWriteAction, ConcurrentWriteProfileHandleBlocking, ConcurrentWriteSelectorHandle}, DataError
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::{
    app::{FilePackageProvider, GetManagerApi, GetSimpleBackendConfig, GetTileMap, PerfCounterDataProvider, SignInWith}, email::{EmailData, EmailDataProvider, EmailError}, file_package::FilePackageManager, manager_client::ManagerApiManager, map::TileMapManager, perf::PerfCounterManagerData, sign_in_with::SignInWithManager
};
use simple_backend_config::SimpleBackendConfig;

pub use crate::app::*;
use super::S;

// Server common

impl EventManagerProvider for S {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(self.database.cache(), &self.push_notification_sender)
    }
}

impl GetAccounts for S {
    async fn get_internal_id(&self, id: AccountId) -> error_stack::Result<AccountIdInternal, DataError> {
        self.database
            .account_id_manager()
            .get_internal_id(id)
            .await
            .map_err(|e| e.into_report())
    }

    async fn get_internal_id_optional(&self, id: AccountId) -> Option<AccountIdInternal> {
        self.database
            .account_id_manager()
            .get_internal_id_optional(id)
            .await
    }
}

impl ReadDynamicConfig for S {
    async fn read_config(&self) -> error_stack::Result<BackendConfig, ConfigFileError> {
        let config = tokio::task::spawn_blocking(ConfigFileDynamic::load_from_current_dir)
            .await
            .change_context(ConfigFileError::LoadConfig)??;

        Ok(config.backend_config)
    }
}

impl BackendVersionProvider for S {
    fn backend_version(&self) -> BackendVersion {
        BackendVersion {
            backend_code_version: self
                .simple_backend_config()
                .backend_code_version()
                .to_string(),
            backend_version: self
                .simple_backend_config()
                .backend_semver_version()
                .to_string(),
            protocol_version: "1.0.0".to_string(),
        }
    }
}

impl GetConfig for S {
    fn config(&self) -> &Config {
        &self.config
    }
    fn config_arc(&self) -> std::sync::Arc<Config> {
        self.config.clone()
    }
}

impl WriteDynamicConfig for S {
    async fn write_config(
        &self,
        config: BackendConfig,
    ) -> error_stack::Result<(), ConfigFileError> {
        tokio::task::spawn_blocking(move || {
            if let Some(bots) = config.bots {
                ConfigFileDynamic::edit_bot_config_from_current_dir(bots)?
            }

            error_stack::Result::<(), ConfigFileError>::Ok(())
        })
        .await
        .change_context(ConfigFileError::LoadConfig)??;

        Ok(())
    }
}

impl PushNotificationStateProvider for S {
    async fn get_push_notification_state_info_and_add_notification_value(
        &self,
        account_id: AccountIdInternal,
    ) -> error_stack::Result<PushNotificationStateInfoWithFlags, PushNotificationError> {
        let flags = self
            .read()
            .common()
            .cached_pending_notification_flags(account_id)
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::ReadingNotificationFlagsFromCacheFailed)?;

        if flags.is_empty() {
            return Ok(PushNotificationStateInfoWithFlags::EmptyFlags);
        }

        let info = db_write_raw!(self, move |cmds| {
            cmds.chat()
                .push_notifications()
                .get_push_notification_state_info_and_add_notification_value(
                    account_id,
                    flags.into(),
                )
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)?;

        Ok(PushNotificationStateInfoWithFlags::WithFlags {
            info,
            flags,
        })
    }

    async fn enable_push_notification_sent_flag(
        &self,
        account_id: AccountIdInternal,
    ) -> error_stack::Result<(), PushNotificationError> {
        db_write_raw!(self, move |cmds| {
            cmds.chat()
                .push_notifications()
                .enable_push_notification_sent_flag(account_id)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)?;

        Ok(())
    }

    async fn remove_device_token(
        &self,
        account_id: AccountIdInternal,
    ) -> error_stack::Result<(), PushNotificationError> {
        db_write_raw!(self, move |cmds| {
            cmds.chat()
                .push_notifications()
                .remove_fcm_device_token(account_id)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(PushNotificationError::RemoveDeviceTokenFailed)
    }

    async fn remove_specific_notification_flags_from_cache(
        &self,
        account_id: AccountIdInternal,
        flags: PendingNotificationFlags,
    ) -> error_stack::Result<(), PushNotificationError> {
        self.database.cache().write_cache(account_id, move |entry| {
            entry.common.pending_notification_flags -= flags;
            Ok(())
        })
        .await
        .map_err(|e| e.into_error())
        .change_context(PushNotificationError::RemoveSpecificNotificationFlagsFromCacheFailed)
    }

    async fn save_current_non_empty_notification_flags_from_cache_to_database(
        &self,
    ) -> error_stack::Result<(), PushNotificationError> {
        let account_ids = self.read()
            .account()
            .account_ids_internal_vec()
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::SaveToDatabaseFailed)?;

        for account_id in account_ids {
            db_write_raw!(self, move |cmds| {
                cmds.chat()
                    .push_notifications()
                    .save_current_non_empty_notification_flags_from_cache_to_database(account_id)
                    .await
            })
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::SaveToDatabaseFailed)?;
        }

        Ok(())
    }
}

impl ResetPushNotificationTokens for S {
    async fn reset_push_notification_tokens(
        &self,
        account_id: AccountIdInternal,
    ) -> server_common::result::Result<(), DataError> {
        db_write_raw!(self, move |cmds| {
            cmds.chat().push_notifications().remove_fcm_device_token_and_pending_notification_token(account_id).await
        })
        .await
    }
}

impl LatestPublicKeysInfo for S {
    async fn latest_public_keys_info(
        &self,
        account_id: AccountIdInternal,
    ) -> server_common::result::Result<Vec<PublicKeyIdAndVersion>, DataError> {
        self.read().chat().get_latest_public_keys_info(account_id).await
    }
}

// Server data

impl WriteData for S {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = server_common::result::Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        self.write_queue.write(cmd).await
    }

    // async fn write<
    //     CmdResult: Send + 'static,
    //     Cmd: Future<Output = server_common::result::Result<CmdResult, DataError>> + Send,
    //     GetCmd,
    // >(
    //     &self,
    //     write_cmd: GetCmd,
    // ) -> server_common::result::Result<CmdResult, DataError> where GetCmd: FnOnce(SyncWriteHandleRef<'_>) -> Cmd + Send + 'static,  {
    //     self.write_queue.write_with_ref_handle(write_cmd).await
    // }

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = ConcurrentWriteAction<CmdResult>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteSelectorHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        self.write_queue.concurrent_write(account, cmd).await
    }

    async fn concurrent_write_profile_blocking<
        CmdResult: Send + 'static,
        WriteCmd: FnOnce(ConcurrentWriteProfileHandleBlocking) -> CmdResult + Send + 'static,
    >(
        &self,
        account: AccountId,
        write_cmd: WriteCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        self.write_queue.concurrent_write_profile_blocking(account, write_cmd).await
    }
}

impl ReadData for S {
    fn read(&self) -> &RouterDatabaseReadHandle {
        &self.database
    }
}

// Server data profile

impl ProfileStatisticsCacheProvider for S {
    fn profile_statistics_cache(&self) -> &server_data::statistics::ProfileStatisticsCache {
        &self.profile_statistics_cache
    }
}

// Server API

impl StateBase for S {}

impl GetInternalApi for S {
    fn internal_api_client(&self) -> &InternalApiClient {
        &self.internal_api
    }
}

impl GetAccessTokens for S {
    async fn access_token_exists(&self, token: &AccessToken) -> Option<AccountIdInternal> {
        self.database
            .access_token_manager()
            .access_token_exists(token)
            .await
    }

    async fn access_token_and_connection_exists(
        &self,
        token: &AccessToken,
        connection: SocketAddr,
    ) -> Option<(AccountIdInternal, Permissions, AccountState)> {
        self.database
            .access_token_manager()
            .access_token_and_connection_exists(token, connection)
            .await
    }
}

impl ContentProcessingProvider for S {
    fn content_processing(&self) -> &ContentProcessingManagerData {
        &self.content_processing
    }
}

impl ValidateModerationRequest for S {
    async fn media_check_moderation_request_for_account(
        &self,
        account_id: AccountIdInternal,
    ) -> server_common::result::Result<(), server_common::internal_api::InternalApiError> {
        crate::internal_api::media::media_check_moderation_request_for_account(self, account_id)
            .await
    }
}

impl IsMatch for S {
    async fn is_match(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> server_common::result::Result<bool, DataError> {
        let interaction = self.read().chat().account_interaction(account0, account1).await?;
        if let Some(interaction) = interaction {
            Ok(interaction.is_match() && !interaction.is_blocked())
        } else {
            Ok(false)
        }
    }
}

// Simple backend

impl SignInWith for S {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        &self.simple_backend_state.sign_in_with
    }
}

impl GetManagerApi for S {
    fn manager_api(&self) -> ManagerApiManager {
        ManagerApiManager::new(&self.simple_backend_state.manager_api)
    }
}

impl GetSimpleBackendConfig for S {
    fn simple_backend_config(&self) -> &SimpleBackendConfig {
        &self.simple_backend_state.config
    }
}

impl GetTileMap for S {
    fn tile_map(&self) -> &TileMapManager {
        &self.simple_backend_state.tile_map
    }
}

impl PerfCounterDataProvider for S {
    fn perf_counter_data(&self) -> &PerfCounterManagerData {
        &self.simple_backend_state.perf_data
    }
}

impl FilePackageProvider for S {
    fn file_package(&self) -> &FilePackageManager {
        &self.simple_backend_state.file_packages
    }
}

impl EmailDataProvider<AccountIdInternal, EmailMessages> for S {
    async fn get_email_data(
        &self,
        receiver: AccountIdInternal,
        message: EmailMessages,
    ) -> error_stack::Result<Option<EmailData>, simple_backend::email::EmailError> {
        let data = self
            .read()
            .account()
            .account_data(receiver)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        if data.email.is_none() {
            return Ok(None);
        }

        let email = if let Some(email) = data.email {
            if email.0.ends_with("example.com") {
                return Ok(None);
            }

            email.0
        } else {
            return Ok(None)
        };

        let email_content = self
            .config()
            .email_content()
            .ok_or(EmailError::GettingEmailDataFailed)
            .attach_printable("Email content not configured")?;

        let email_content = email_content.email
            .iter()
            .find(|e| e.message_type == message)
            .ok_or(EmailError::GettingEmailDataFailed)
            .attach_printable(format!("Email content for {:?} is not configured", message))?;

        let email_data = EmailData {
            email_address: email,
            subject: email_content.subject.clone(),
            body: email_content.body.clone(),
        };

        Ok(Some(email_data))
    }

    async fn mark_as_sent(
        &self,
        receiver: AccountIdInternal,
        message: EmailMessages,
    ) -> error_stack::Result<(), simple_backend::email::EmailError> {
        db_write_raw!(self, move |cmds| {
            cmds.account().email().mark_email_as_sent(receiver, message).await
        })
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::MarkAsSentFailed)

    }
}
