#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

//! This crate provides a wrapper for the internal API of the server.
//! Prevents exposing api_client crate model types to server code.

use api_client::apis::{
    account_internal_api,
    media_internal_api::{self},
};
pub use api_client::apis::{configuration::Configuration, Error};
use model::{
    AccessToken, Account, AccountId, AccountIdInternal, AccountState, BooleanSetting, Capabilities,
    Profile,
};

pub use crate::{
    account_internal_api::{CheckAccessTokenError, InternalGetAccountStateError},
    media_internal_api::InternalGetCheckModerationRequestForAccountError,
};

/// Wrapper for server internal API with correct model types.
pub struct InternalApi;

impl InternalApi {
    pub async fn check_access_token(
        configuration: &Configuration,
        token: AccessToken,
    ) -> Result<AccountId, Error<CheckAccessTokenError>> {
        account_internal_api::check_access_token(
            configuration,
            api_client::models::AccessToken {
                access_token: token.into_string(),
            },
        )
        .await
        .map(|data| AccountId::new(data.account_id))
    }

    pub async fn get_account_state(
        configuration: &Configuration,
        account_id: AccountId,
    ) -> Result<Account, Error<InternalGetAccountStateError>> {
        let account = account_internal_api::internal_get_account_state(
            configuration,
            &account_id.to_string(),
        )
        .await?;

        let state = match account.state {
            api_client::models::AccountState::InitialSetup => AccountState::InitialSetup,
            api_client::models::AccountState::Normal => AccountState::Normal,
            api_client::models::AccountState::Banned => AccountState::Banned,
            api_client::models::AccountState::PendingDeletion => AccountState::PendingDeletion,
        };

        let capabilities_string =
            serde_json::to_string(&account.capabilities).map_err(|e| Error::Serde(e))?;
        let capabilities: Capabilities =
            serde_json::from_str(&capabilities_string).map_err(|e| Error::Serde(e))?;

        Ok(Account::new_from(state, capabilities))
    }

    pub async fn media_check_moderation_request_for_account(
        configuration: &Configuration,
        account_id: AccountId,
    ) -> Result<(), Error<InternalGetCheckModerationRequestForAccountError>> {
        media_internal_api::internal_get_check_moderation_request_for_account(
            configuration,
            &account_id.to_string(),
        )
        .await
    }

    pub async fn profile_api_set_profile_visiblity(
        _configuration: &Configuration,
        _account_id: AccountIdInternal,
        _boolean_setting: BooleanSetting,
    ) -> Result<(), ()> {
        // TODO: Request internal profile api
        Ok(())
    }

    pub async fn media_api_profile_visiblity(
        _configuration: &Configuration,
        _account_id: AccountIdInternal,
        _boolean_setting: BooleanSetting,
        _current_profile: Profile,
    ) -> Result<(), ()> {
        // TODO: request to internal media API
        Ok(())
    }
}
