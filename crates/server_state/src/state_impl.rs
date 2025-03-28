use std::net::SocketAddr;

use config::{file::ConfigFileError, file_dynamic::ConfigFileDynamic, Config};
use error_stack::ResultExt;
use futures::Future;
use manager_model::ServerEventType;
use model::{
    AccessToken, AccountId, AccountIdInternal, AccountState, BackendConfig, BackendVersion, EventToClientInternal, Permissions, ScheduledMaintenanceStatus
};
use server_data::{
    content_processing::ContentProcessingManagerData,
    db_manager::RouterDatabaseReadHandle,
    event::EventManagerWithCacheReference,
    write_commands::WriteCmds,
    write_concurrent::{
        ConcurrentWriteAction, ConcurrentWriteProfileHandleBlocking, ConcurrentWriteSelectorHandle,
    },
    DataError,
};
use simple_backend::{
    app::{
        FilePackageProvider, GetManagerApi, GetSimpleBackendConfig, GetTileMap,
        PerfCounterDataProvider, SignInWith,
    }, file_package::FilePackageManager, manager_client::{ManagerApiClient, ManagerEventHandler}, map::TileMapManager, perf::PerfMetricsManagerData, sign_in_with::SignInWithManager
};
use simple_backend_config::SimpleBackendConfig;

use super::S;
pub use crate::app::*;
use crate::{api_usage::ApiUsageTracker, client_version::ClientVersionTracker, internal_api::InternalApiClient};

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
        self.state.database
            .account_id_manager()
            .get_internal_id(id)
            .await
            .map_err(|e| e.into_report())
    }

    async fn get_internal_id_optional(&self, id: AccountId) -> Option<AccountIdInternal> {
        self.state.database
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
        &self.state.config
    }
    fn config_arc(&self) -> std::sync::Arc<Config> {
        self.state.config.clone()
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
                    config.bots,
                    config.remote_bot_login,
                )?
            }

            error_stack::Result::<(), ConfigFileError>::Ok(())
        })
        .await
        .change_context(ConfigFileError::LoadConfig)??;

        Ok(())
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
        self.state.write_queue
            .concurrent_write_profile_blocking(account, write_cmd)
            .await
    }
}

impl ReadData for S {
    fn read(&self) -> &RouterDatabaseReadHandle {
        &self.state.database
    }
}

// Server data profile

impl ProfileStatisticsCacheProvider for S {
    fn profile_statistics_cache(&self) -> &server_data::statistics::ProfileStatisticsCache {
        &self.state.profile_statistics_cache
    }
}

// Server API

impl GetInternalApi for S {
    fn internal_api_client(&self) -> &InternalApiClient {
        &self.state.internal_api
    }
}

impl GetAccessTokens for S {
    async fn access_token_exists(&self, token: &AccessToken) -> Option<AccountIdInternal> {
        self.state.database
            .access_token_manager()
            .access_token_exists(token)
            .await
    }

    async fn access_token_and_connection_exists(
        &self,
        token: &AccessToken,
        connection: SocketAddr,
    ) -> Option<(AccountIdInternal, Permissions, AccountState)> {
        self.state.database
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
}

impl FilePackageProvider for S {
    fn file_package(&self) -> &FilePackageManager {
        &self.state.simple_backend_state.file_packages
    }
}

impl ManagerEventHandler for S {
    async fn handle(&self, event: &ServerEventType) {
        match event {
            ServerEventType::MaintenanceSchedulingStatus(status) => {
                let status = ScheduledMaintenanceStatus {
                    scheduled_maintenance: status.map(|v| v.0)
                };
                self.event_manager().send_connected_event_to_logged_in_clients(
                    EventToClientInternal::ScheduledMaintenanceStatus(status),
                ).await
            }
        }
    }
}
