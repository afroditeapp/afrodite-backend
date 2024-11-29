use std::net::SocketAddr;

use axum::extract::ws::WebSocket;
use model::{
    AccessToken, AccountIdInternal, AccountState, PendingNotification, PendingNotificationWithData, Permissions, PublicKeyIdAndVersion, SyncDataVersionFromClient
};
use server_common::internal_api::InternalApiError;
use server_data::{content_processing::ContentProcessingManagerData, DataError};
use crate::{internal_api::InternalApiClient, utils::StatusCode, websocket::WebSocketError};

pub use server_data::app::*;

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

pub trait StateBase: Send + Sync + Clone + 'static {}

pub trait ValidateModerationRequest: GetConfig + ReadData + GetInternalApi {
    fn media_check_moderation_request_for_account(
        &self,
        account_id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), InternalApiError>> + Send;
}

pub trait CompleteInitialSetupCmd: ReadData + WriteData + GetInternalApi + GetConfig + ValidateModerationRequest {
    fn complete_initial_setup(
        &self,
        account_id: AccountIdInternal,
    ) -> impl std::future::Future<Output = std::result::Result<(), StatusCode>> + Send;
}

pub trait ConnectionTools: StateBase + WriteData + ReadData + GetConfig {
    fn reset_pending_notification(
        &self,
        id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), WebSocketError>> + Send;

    fn send_new_messages_event_if_needed(
        &self,
        socket: &mut WebSocket,
        id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), WebSocketError>> + Send;

    fn sync_data_with_client_if_needed(
        &self,
        socket: &mut WebSocket,
        id: AccountIdInternal,
        sync_versions: Vec<SyncDataVersionFromClient>,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), WebSocketError>> + Send;
}

pub trait ResetPushNotificationTokens: StateBase + WriteData {
    fn reset_push_notification_tokens(
        &self,
        id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), DataError>> + Send;
}

pub trait IsMatch: StateBase + ReadData {
    /// Account interaction is in match state and there is no one or two way block.
    fn is_match(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<bool, DataError>> + Send;
}

pub trait LatestPublicKeysInfo: StateBase + WriteData {
    fn latest_public_keys_info(
        &self,
        id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<Vec<PublicKeyIdAndVersion>, DataError>> + Send;
}

pub trait GetPushNotificationData: StateBase + ReadData {
    fn get_push_notification_data(
        &self,
        id: AccountIdInternal,
        pending_notification: PendingNotification,
    ) -> impl std::future::Future<Output = PendingNotificationWithData> + Send;
}
