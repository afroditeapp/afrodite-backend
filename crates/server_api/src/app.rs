use std::net::SocketAddr;

use model::{AccessToken, AccountIdInternal, AccountState, Capabilities, EmailAddress, SignInWithInfo};
use model::{AccessibleAccount, DemoModeConfirmLoginResult, DemoModeId, DemoModeLoginResult, DemoModeLoginToken, DemoModePassword, DemoModeToken};
use server_common::internal_api::InternalApiError;
pub use server_data::app::*;
use server_data::content_processing::ContentProcessingManagerData;
use server_data::DataError;

use crate::{internal_api::InternalApiClient, utils::StatusCode};

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

pub trait StateBase: Send + Sync + Clone + 'static {}


pub trait ValidateModerationRequest {
    fn media_check_moderation_request_for_account<
        S: GetConfig + ReadData + GetInternalApi,
    >(
        &self,
        state: &S,
        account_id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), InternalApiError>> + Send;
}

pub trait RegisteringCmd {
    fn register_impl<S: WriteData>(
        &self,
        state: &S,
        sign_in_with: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> impl std::future::Future<Output = Result<AccountIdInternal, StatusCode>> + Send;
}

pub trait DemoModeManagerProvider {
    fn stage0_login(
        &self,
        password: DemoModePassword,
    ) -> impl std::future::Future<Output = server_common::result::Result<DemoModeLoginResult, DataError>> + Send;

    fn stage1_login(
        &self,
        password: DemoModePassword,
        token: DemoModeLoginToken,
    ) -> impl std::future::Future<Output = server_common::result::Result<DemoModeConfirmLoginResult, DataError>> + Send;

    fn demo_mode_token_exists(
        &self,
        token: &DemoModeToken,
    ) -> impl std::future::Future<Output = server_common::result::Result<DemoModeId, DataError>> + Send;

    fn accessible_accounts_if_token_valid<S: GetConfig + GetAccounts + ReadData>(
        &self,
        state: &S,
        token: &DemoModeToken,
    ) -> impl std::future::Future<Output = server_common::result::Result<Vec<AccessibleAccount>, DataError>> + Send;
}
