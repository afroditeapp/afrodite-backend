use std::net::SocketAddr;

use model::{AccessToken, AccountIdInternal, AccountState, Capabilities};
pub use server_data::app::*;
use server_data::{content_processing::ContentProcessingManagerData, demo::DemoModeManager};

use crate::internal_api::InternalApiClient;

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
    ) -> impl std::future::Future<Output = Option<(AccountIdInternal, Capabilities, AccountState)>> + Send;
}

pub trait ContentProcessingProvider {
    fn content_processing(&self) -> &ContentProcessingManagerData;
}

pub trait DemoModeManagerProvider {
    fn demo_mode(&self) -> &DemoModeManager;
}

pub trait StateBase: Send + Sync + Clone + 'static {}
