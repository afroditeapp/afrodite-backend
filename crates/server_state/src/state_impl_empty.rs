use std::net::SocketAddr;

use config::{file::ConfigFileError, Config};
use futures::Future;
use model::{
    AccessToken, AccountId, AccountIdInternal, AccountState, BackendConfig, BackendVersion, EmailAddress, EmailMessages, PendingNotification, PendingNotificationFlags, PendingNotificationWithData, Permissions, PublicKeyIdAndVersion, PushNotificationStateInfoWithFlags, SignInWithInfo
};
pub use server_api::app::*;
use server_api::{internal_api::InternalApiClient, utils::StatusCode};
use server_common::push_notifications::{PushNotificationError, PushNotificationStateProvider};
use server_data::{
    content_processing::ContentProcessingManagerData,
    event::EventManagerWithCacheReference,
    read::ReadCommandsContainer,
    write_commands::WriteCmds,
    write_concurrent::{ConcurrentWriteAction, ConcurrentWriteProfileHandleBlocking, ConcurrentWriteSelectorHandle},
    DataError
};
use server_data_profile::app::ProfileStatisticsCacheProvider;
use simple_backend::{
    app::{FilePackageProvider, GetManagerApi, GetSimpleBackendConfig, GetTileMap, PerfCounterDataProvider, SignInWith}, email::{EmailData, EmailDataProvider}, file_package::FilePackageManager, manager_client::ManagerApiManager, map::TileMapManager, perf::PerfCounterManagerData, sign_in_with::SignInWithManager
};
use simple_backend_config::SimpleBackendConfig;

use super::E;

// Server common

impl EventManagerProvider for E {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_> {
        unimplemented!()
    }
}

impl GetAccounts for E {
    async fn get_internal_id(&self, _id: AccountId) -> error_stack::Result<AccountIdInternal, DataError> {
        unimplemented!()
    }
    async fn get_internal_id_optional(&self, _id: AccountId) -> Option<AccountIdInternal> {
        unimplemented!()
    }
}

impl ReadDynamicConfig for E {
    async fn read_config(&self) -> error_stack::Result<BackendConfig, ConfigFileError> {
        unimplemented!()
    }
}

impl BackendVersionProvider for E {
    fn backend_version(&self) -> BackendVersion {
        unimplemented!()
    }
}

impl GetConfig for E {
    fn config(&self) -> &Config {
        unimplemented!()
    }
}

impl WriteDynamicConfig for E {
    async fn write_config(
        &self,
        _config: BackendConfig,
    ) -> error_stack::Result<(), ConfigFileError> {
        unimplemented!()
    }
}

impl PushNotificationStateProvider for E {
    async fn get_push_notification_state_info_and_add_notification_value(
        &self,
        _account_id: AccountIdInternal,
    ) -> error_stack::Result<PushNotificationStateInfoWithFlags, PushNotificationError> {
        unimplemented!()
    }

    async fn enable_push_notification_sent_flag(
        &self,
        _account_id: AccountIdInternal,
    ) -> error_stack::Result<(), PushNotificationError> {
        unimplemented!()
    }

    async fn remove_device_token(
        &self,
        _account_id: AccountIdInternal,
    ) -> error_stack::Result<(), PushNotificationError> {
        unimplemented!()
    }

    async fn remove_specific_notification_flags_from_cache(
        &self,
        _account_id: AccountIdInternal,
        _flags: PendingNotificationFlags,
    ) -> error_stack::Result<(), PushNotificationError> {
        unimplemented!()
    }

    async fn save_current_non_empty_notification_flags_from_cache_to_database(
        &self,
    ) -> error_stack::Result<(), PushNotificationError> {
        unimplemented!()
    }
}

impl ResetPushNotificationTokens for E {
    async fn reset_push_notification_tokens(
        &self,
        _account_id: AccountIdInternal,
    ) -> server_common::result::Result<(), DataError> {
        unimplemented!()
    }
}

impl LatestPublicKeysInfo for E {
    async fn latest_public_keys_info(
        &self,
        _account_id: AccountIdInternal,
    ) -> server_common::result::Result<Vec<PublicKeyIdAndVersion>, DataError> {
        unimplemented!()
    }
}

// Server data

impl WriteData for E {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = server_common::result::Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        _cmd: GetCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        unimplemented!()
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
        _account: AccountId,
        _cmd: GetCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        unimplemented!()
    }

    async fn concurrent_write_profile_blocking<
        CmdResult: Send + 'static,
        WriteCmd: FnOnce(ConcurrentWriteProfileHandleBlocking) -> CmdResult + Send + 'static,
    >(
        &self,
        _account: AccountId,
        _write_cmd: WriteCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        unimplemented!()
    }
}

impl ReadData for E {
    fn read(&self) -> ReadCommandsContainer {
        unimplemented!()
    }
}

// Server data profile

impl ProfileStatisticsCacheProvider for E {
    fn profile_statistics_cache(&self) -> &server_data_profile::statistics::ProfileStatisticsCache {
        unimplemented!()
    }
}

// Server API

impl StateBase for E {}

impl GetInternalApi for E {
    fn internal_api_client(&self) -> &InternalApiClient {
        unimplemented!()
    }
}

impl GetAccessTokens for E {
    async fn access_token_exists(&self, _token: &AccessToken) -> Option<AccountIdInternal> {
        unimplemented!()
    }

    async fn access_token_and_connection_exists(
        &self,
        _token: &AccessToken,
        _connection: SocketAddr,
    ) -> Option<(AccountIdInternal, Permissions, AccountState)> {
        unimplemented!()
    }
}

impl ContentProcessingProvider for E {
    fn content_processing(&self) -> &ContentProcessingManagerData {
        unimplemented!()
    }
}

impl DemoModeManagerProvider for E {
    async fn stage0_login(
        &self,
        _password: model::DemoModePassword,
    ) -> error_stack::Result<model::DemoModeLoginResult, DataError> {
        unimplemented!()
    }

    async fn stage1_login(
        &self,
        _password: model::DemoModePassword,
        _token: model::DemoModeLoginToken,
    ) -> error_stack::Result<model::DemoModeConfirmLoginResult, DataError> {
        unimplemented!()
    }

    async fn demo_mode_token_exists(
        &self,
        _token: &model::DemoModeToken,
    ) -> error_stack::Result<model::DemoModeId, DataError> {
        unimplemented!()
    }

    async fn accessible_accounts_if_token_valid<
        S: StateBase + GetConfig + GetAccounts + ReadData,
    >(
        &self,
        _state: &S,
        _token: &model::DemoModeToken,
    ) -> server_common::result::Result<Vec<model::AccessibleAccount>, DataError> {
        unimplemented!()
    }
}

impl RegisteringCmd for E {
    async fn register_impl(
        &self,
        _sign_in_with: SignInWithInfo,
        _email: Option<EmailAddress>,
    ) -> std::result::Result<AccountIdInternal, StatusCode> {
        unimplemented!()
    }
}

impl ValidateModerationRequest for E {
    async fn media_check_moderation_request_for_account(
        &self,
        _account_id: AccountIdInternal,
    ) -> server_common::result::Result<(), server_common::internal_api::InternalApiError> {
        unimplemented!()
    }
}

impl CompleteInitialSetupCmd for E {
    async fn complete_initial_setup(
        &self,
        _id: AccountIdInternal,
    ) -> std::result::Result<(), StatusCode> {
        unimplemented!()
    }
}


impl IsMatch for E {
    async fn is_match(
        &self,
        _account0: AccountIdInternal,
        _account1: AccountIdInternal,
    ) -> server_common::result::Result<bool, DataError> {
        unimplemented!()
    }
}

impl UpdateUnlimitedLikes for E {
    async fn update_unlimited_likes(
        &self,
        _id: AccountIdInternal,
        _unlimited_likes: bool,
    ) -> server_common::result::Result<(), DataError> {
        unimplemented!()
    }
}


impl GetPushNotificationData for E {
    async fn get_push_notification_data(
        &self,
        _id: AccountIdInternal,
        _pending_notification: PendingNotification,
    ) -> PendingNotificationWithData {
        unimplemented!()
    }
}

// Simple backend

impl SignInWith for E {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        unimplemented!()
    }
}

impl GetManagerApi for E {
    fn manager_api(&self) -> ManagerApiManager {
        unimplemented!()
    }
}

impl GetSimpleBackendConfig for E {
    fn simple_backend_config(&self) -> &SimpleBackendConfig {
        unimplemented!()
    }
}

impl GetTileMap for E {
    fn tile_map(&self) -> &TileMapManager {
        unimplemented!()
    }
}

impl PerfCounterDataProvider for E {
    fn perf_counter_data(&self) -> &PerfCounterManagerData {
        unimplemented!()
    }
}

impl FilePackageProvider for E {
    fn file_package(&self) -> &FilePackageManager {
        unimplemented!()
    }
}

impl EmailDataProvider<AccountIdInternal, EmailMessages> for E {
    async fn get_email_data(
        &self,
        _receiver: AccountIdInternal,
        _message: EmailMessages,
    ) -> error_stack::Result<Option<EmailData>, simple_backend::email::EmailError> {
        unimplemented!()
    }

    async fn mark_as_sent(
        &self,
        _receiver: AccountIdInternal,
        _message: EmailMessages,
    ) -> error_stack::Result<(), simple_backend::email::EmailError> {
        unimplemented!()
    }
}
