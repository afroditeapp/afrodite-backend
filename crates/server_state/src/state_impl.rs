use std::{net::SocketAddr, sync::Arc};

use config::{Config, file::ConfigFileError, file_dynamic::ConfigFileDynamic};
use error_stack::ResultExt;
use futures::Future;
use model::{
    AccessToken, AccountId, AccountIdInternal, AccountState, BackendConfig, BackendVersion,
    EventToClientInternal, Permissions, ScheduledMaintenanceStatus,
};
use server_data::{
    DataError,
    content_processing::ContentProcessingManagerData,
    db_manager::RouterDatabaseReadHandle,
    event::EventManagerWithCacheReference,
    write_commands::WriteCmds,
    write_concurrent::{
        ConcurrentWriteAction, ConcurrentWriteProfileHandleBlocking, ConcurrentWriteSelectorHandle,
    },
};
use simple_backend::{
    app::{
        FilePackageProvider, GetManagerApi, GetSimpleBackendConfig, GetTileMap,
        IpCountryTrackerProvider, JitsiMeetUrlCreatorProvider, MaxMindDbDataProvider,
        PerfCounterDataProvider, SignInWith,
    },
    file_package::FilePackageManager,
    ip_country::IpCountryTracker,
    jitsi_meet::JitsiMeetUrlCreator,
    manager_client::{ManagerApiClient, ManagerEventHandler},
    map::TileMapManager,
    perf::PerfMetricsManagerData,
    sign_in_with::SignInWithManager,
};
use simple_backend_config::SimpleBackendConfig;

use super::S;
pub use crate::app::*;
use crate::{
    api_limits::ApiLimits, api_usage::ApiUsageTracker, client_version::ClientVersionTracker,
};

// Server common

impl EventManagerProvider for S {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(
            self.state.database.cache_read_write_access(),
            &self.state.push_notification_sender,
        )
    }
}

impl GetAccounts for S {
    async fn get_internal_id(
        &self,
        id: AccountId,
    ) -> error_stack::Result<AccountIdInternal, DataError> {
        self.state
            .database
            .account_id_manager()
            .get_internal_id(id)
            .await
            .map_err(|e| e.into_report())
    }

    async fn get_internal_id_optional(&self, id: AccountId) -> Option<AccountIdInternal> {
        self.state
            .database
            .account_id_manager()
            .get_internal_id_optional(id)
            .await
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
        &self.state.config
    }
    fn config_arc(&self) -> std::sync::Arc<Config> {
        self.state.config.clone()
    }
}

impl ReadDynamicConfig for S {
    async fn read_config(&self) -> error_stack::Result<BackendConfig, ConfigFileError> {
        let config = tokio::task::spawn_blocking(|| ConfigFileDynamic::load_from_current_dir(true))
            .await
            .change_context(ConfigFileError::LoadConfig)??;

        Ok(config.backend_config)
    }

    fn is_remote_bot_login_enabled(&self) -> bool {
        self.state
            .dynamic_config_manager
            .is_remote_bot_login_enabled()
    }
}

impl WriteDynamicConfig for S {
    async fn write_config(
        &self,
        config: BackendConfig,
    ) -> error_stack::Result<(), ConfigFileError> {
        tokio::task::spawn_blocking(move || {
            if BackendConfig::empty() != config {
                ConfigFileDynamic::edit_config_from_current_dir(
                    config.remote_bot_login,
                    config.local_bots.clone().and_then(|v| v.admin),
                    config.local_bots.and_then(|v| v.users),
                )?
            }

            error_stack::Result::<(), ConfigFileError>::Ok(())
        })
        .await
        .change_context(ConfigFileError::LoadConfig)??;

        self.state.dynamic_config_manager.reload().await;

        Ok(())
    }

    fn set_remote_bot_login_enabled(&self, value: bool) {
        self.state
            .dynamic_config_manager
            .set_remote_bot_login_enabled(value);
    }

    async fn reload_dynamic_config(&self) {
        self.state.dynamic_config_manager.reload().await;
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
        self.state.write_queue.write(cmd).await
    }

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = ConcurrentWriteAction<CmdResult>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteSelectorHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        self.state.write_queue.concurrent_write(account, cmd).await
    }

    async fn concurrent_write_profile_blocking<
        CmdResult: Send + 'static,
        WriteCmd: FnOnce(ConcurrentWriteProfileHandleBlocking) -> CmdResult + Send + 'static,
    >(
        &self,
        account: AccountId,
        write_cmd: WriteCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        self.state
            .write_queue
            .concurrent_write_profile_blocking(account, write_cmd)
            .await
    }
}

impl ReadData for S {
    fn read(&self) -> &RouterDatabaseReadHandle {
        &self.state.database
    }
}

impl ProfileStatisticsCacheProvider for S {
    fn profile_statistics_cache(&self) -> &server_data::statistics::ProfileStatisticsCache {
        &self.state.profile_statistics_cache
    }
}

impl DataExportManagerDataProvider for S {
    fn data_export(&self) -> &server_data::data_export::DataExportManagerData {
        &self.state.data_export
    }
}

// Server API

impl GetAccessTokens for S {
    async fn access_token_exists(&self, token: &AccessToken) -> Option<AccountIdInternal> {
        self.state
            .database
            .access_token_manager()
            .access_token_exists(token)
            .await
    }

    async fn access_token_and_connection_exists(
        &self,
        token: &AccessToken,
        connection: SocketAddr,
    ) -> Option<(AccountIdInternal, Permissions, AccountState)> {
        self.state
            .database
            .access_token_manager()
            .access_token_and_connection_exists(token, connection)
            .await
    }
}

impl ContentProcessingProvider for S {
    fn content_processing(&self) -> &ContentProcessingManagerData {
        &self.state.content_processing
    }
}

impl ClientVersionTrackerProvider for S {
    fn client_version_tracker(&self) -> &ClientVersionTracker {
        &self.state.client_version_tracker
    }
}

impl ApiUsageTrackerProvider for S {
    fn api_usage_tracker(&self) -> &ApiUsageTracker {
        &self.state.api_usage_tracker
    }
}

impl IpAddressUsageTrackerProvider for S {
    fn ip_address_usage_tracker(&self) -> &crate::ip_address::IpAddressUsageTracker {
        &self.state.ip_address_usage_tracker
    }
}

impl DataSignerProvider for S {
    fn data_signer(&self) -> &crate::data_signer::DataSigner {
        &self.state.data_signer
    }
}

impl AdminNotificationProvider for S {
    fn admin_notification(&self) -> &crate::admin_notifications::AdminNotificationManagerData {
        &self.state.admin_notification
    }
}

impl ApiLimitsProvider for S {
    fn api_limits(&self, account_id: AccountIdInternal) -> crate::api_limits::ApiLimits {
        ApiLimits::new(
            self.read().cache_read_write_access(),
            self.config(),
            account_id,
        )
    }
}

// Simple backend

impl SignInWith for S {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        &self.state.simple_backend_state.sign_in_with
    }
}

impl GetManagerApi for S {
    fn manager_api_client(&self) -> &ManagerApiClient {
        &self.state.simple_backend_state.manager_api
    }
}

impl GetSimpleBackendConfig for S {
    fn simple_backend_config(&self) -> &SimpleBackendConfig {
        &self.state.simple_backend_state.config
    }
}

impl GetTileMap for S {
    fn tile_map(&self) -> &TileMapManager {
        &self.state.simple_backend_state.tile_map
    }
}

impl PerfCounterDataProvider for S {
    fn perf_counter_data(&self) -> &PerfMetricsManagerData {
        &self.state.simple_backend_state.perf_data
    }

    fn perf_counter_data_arc(&self) -> Arc<PerfMetricsManagerData> {
        self.state.simple_backend_state.perf_data.clone()
    }
}

impl FilePackageProvider for S {
    fn file_package(&self) -> &FilePackageManager {
        &self.state.simple_backend_state.file_packages
    }
}

impl MaxMindDbDataProvider for S {
    fn maxmind_db(&self) -> &simple_backend::maxmind_db::MaxMindDbManagerData {
        &self.state.simple_backend_state.maxmind_db
    }
}

impl JitsiMeetUrlCreatorProvider for S {
    fn jitsi_meet_url_creator(&self) -> JitsiMeetUrlCreator {
        JitsiMeetUrlCreator::new(&self.state.simple_backend_state.config)
    }
}

impl ManagerEventHandler for S {
    async fn send_maintenance_status(&self, status: ScheduledMaintenanceStatus) {
        self.event_manager()
            .send_connected_event_to_logged_in_clients(
                EventToClientInternal::ScheduledMaintenanceStatus(status),
            )
            .await
    }
}

impl IpCountryTrackerProvider for S {
    fn ip_country_tracker(&self) -> &IpCountryTracker {
        &self.state.simple_backend_state.ip_country
    }
}
