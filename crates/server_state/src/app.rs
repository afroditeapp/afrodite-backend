use std::net::SocketAddr;

use model::{AccessToken, AccountIdInternal, AccountState, Permissions};
pub use server_data::app::*;
use server_data::{content_processing::ContentProcessingManagerData, DataError};

use crate::{admin_notifications::AdminNotificationManagerData, api_usage::ApiUsageTracker, client_version::ClientVersionTracker, data_signer::DataSigner, internal_api::InternalApiClient, ip_address::IpAddressUsageTracker};

// TODO(prod): Move push notifications to common

pub trait GetInternalApi {
    fn internal_api_client(&self) -> &InternalApiClient;
}

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
