use std::net::SocketAddr;

use model::{
    AccessToken, AccessibleAccount, AccountIdInternal, AccountState, Capabilities,
    DemoModeConfirmLoginResult, DemoModeId, DemoModeLoginResult, DemoModeLoginToken,
    DemoModePassword, DemoModeToken, EmailAddress, SignInWithInfo,
};
use server_common::internal_api::InternalApiError;
pub use server_data::app::*;
use server_data::{content_processing::ContentProcessingManagerData, DataError};

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

pub trait ValidateModerationRequest: GetConfig + ReadData + GetInternalApi {
    fn media_check_moderation_request_for_account(
        &self,
        account_id: AccountIdInternal,
    ) -> impl std::future::Future<Output = server_common::result::Result<(), InternalApiError>> + Send;
}

pub trait RegisteringCmd: WriteData {
    fn register_impl(
        &self,
        sign_in_with: SignInWithInfo,
        email: Option<EmailAddress>,
    ) -> impl std::future::Future<Output = Result<AccountIdInternal, StatusCode>> + Send;
}

pub trait DemoModeManagerProvider: StateBase {
    fn stage0_login(
        &self,
        password: DemoModePassword,
    ) -> impl std::future::Future<Output = error_stack::Result<DemoModeLoginResult, DataError>> + Send;

    fn stage1_login(
        &self,
        password: DemoModePassword,
        token: DemoModeLoginToken,
    ) -> impl std::future::Future<Output = error_stack::Result<DemoModeConfirmLoginResult, DataError>>
           + Send;

    fn demo_mode_token_exists(
        &self,
        token: &DemoModeToken,
    ) -> impl std::future::Future<Output = error_stack::Result<DemoModeId, DataError>> + Send;

    fn accessible_accounts_if_token_valid<S: StateBase + GetConfig + GetAccounts + ReadData>(
        &self,
        state: &S,
        token: &DemoModeToken,
    ) -> impl std::future::Future<
        Output = server_common::result::Result<Vec<AccessibleAccount>, DataError>,
    > + Send;
}
