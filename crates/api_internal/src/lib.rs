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
