use std::net::SocketAddr;

use model::{AccessToken, AccountIdInternal, AccountState, Permissions};
use server_common::internal_api::InternalApiError;
pub use server_data::app::*;
use server_data::{content_processing::ContentProcessingManagerData, DataError};

use crate::internal_api::InternalApiClient;

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

pub trait ValidateModerationRequest: GetConfig + ReadData + GetInternalApi {
    fn media_check_moderation_request_for_account(
        &self,
        account_id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), InternalApiError>> + Send;
}

pub trait IsMatch: ReadData {
    /// Account interaction is in match state and there is no one or two way block.
    fn is_match(
        &self,
        account0: AccountIdInternal,
        account1: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<bool, DataError>> + Send;
}
