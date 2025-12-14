use std::net::SocketAddr;

use config::file::ConfigFileError;
use model::{AccessToken, AccountIdInternal, AccountState, BackendConfig, Permissions};
pub use server_data::app::*;
use server_data::{DataError, content_processing::ContentProcessingManagerData};

use crate::{
    admin_notifications::AdminNotificationManagerData, api_limits::ApiLimits,
    api_usage::ApiUsageTracker, client_version::ClientVersionTracker, data_signer::DataSigner,
    ip_address::IpAddressUsageTracker,
};

pub trait GetAccessTokens {
    fn access_token_exists(
        &self,
        token: &AccessToken,
    ) -> impl std::future::Future<Output = Option<AccountIdInternal>> + Send;

    /// Check that token and current connection IP and port matches
    /// with WebSocket connection.
    fn access_token_and_connection_exists(
        &self,
        token: &AccessToken,
        connection: SocketAddr,
    ) -> impl std::future::Future<Output = Option<(AccountIdInternal, Permissions, AccountState)>> + Send;
}

pub trait ContentProcessingProvider {
    fn content_processing(&self) -> &ContentProcessingManagerData;
}

pub trait IsMatch: ReadData {
    /// Account interaction is in match state and there is no one or two way block.
    fn is_match(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<bool, DataError>> + Send;
}

pub trait ClientVersionTrackerProvider {
    fn client_version_tracker(&self) -> &ClientVersionTracker;
}

pub trait ApiUsageTrackerProvider {
    fn api_usage_tracker(&self) -> &ApiUsageTracker;
}

pub trait IpAddressUsageTrackerProvider {
    fn ip_address_usage_tracker(&self) -> &IpAddressUsageTracker;
}

pub trait DataSignerProvider {
    fn data_signer(&self) -> &DataSigner;
}

pub trait AdminNotificationProvider {
    fn admin_notification(&self) -> &AdminNotificationManagerData;
}

pub trait ApiLimitsProvider {
    fn api_limits(&self, account_id: AccountIdInternal) -> ApiLimits<'_>;
}

pub trait ReadDynamicConfig {
    async fn read_config(&self) -> error_stack::Result<BackendConfig, ConfigFileError>;

    fn is_remote_bot_login_enabled(&self) -> bool;
}

pub trait WriteDynamicConfig {
    async fn write_config(&self, config: BackendConfig)
    -> error_stack::Result<(), ConfigFileError>;

    fn set_remote_bot_login_enabled(&self, value: bool);

    async fn reload_dynamic_config(&self);
}
