#![deny(unsafe_code)]
#![warn(unused_crate_dependencies)]

//! This crate provides a wrapper for the internal API of the server.
//! Prevents exposing api_client crate model types to server code.

use api_client::apis::{
    accountinternal_api,
    mediainternal_api::{self},
};
pub use api_client::apis::{configuration::Configuration, Error};
use model::{
    Account, AccountIdInternal, AccountIdLight, AccountState, ApiKey, BooleanSetting, Capabilities,
    Profile,
};

pub use crate::{
    accountinternal_api::{CheckApiKeyError, InternalGetAccountStateError},
    mediainternal_api::InternalGetCheckModerationRequestForAccountError,
};

/// Wrapper for server internal API with correct model types.
pub struct InternalApi;

impl InternalApi {
    pub async fn check_api_key(
        configuration: &Configuration,
        key: ApiKey,
    ) -> Result<AccountIdLight, Error<CheckApiKeyError>> {
        accountinternal_api::check_api_key(
            configuration,
            api_client::models::ApiKey {
                api_key: key.into_string(),
            },
        )
        .await
        .map(|data| AccountIdLight::new(data.account_id))
    }

    pub async fn get_account_state(
        configuration: &Configuration,
        account_id: AccountIdLight,
    ) -> Result<Account, Error<InternalGetAccountStateError>> {
        let account =
            accountinternal_api::internal_get_account_state(configuration, &account_id.to_string())
                .await?;

        let state = match account.state {
            api_client::models::AccountState::InitialSetup => AccountState::InitialSetup,
            api_client::models::AccountState::Normal => AccountState::Normal,
            api_client::models::AccountState::Banned => AccountState::Banned,
            api_client::models::AccountState::PendingDeletion => AccountState::PendingDeletion,
        };

        macro_rules! copy_capablities {
            ($account:expr,  $( $name:ident , )* ) => {
                Capabilities {
                    $( $name: $account.capablities.$name.unwrap_or(false), )*
                    ..Capabilities::default()
                }
            };
        }
        // TODO: Add missing capabilities
        let capabilities = copy_capablities!(
            account,
            admin_modify_capablities,
            admin_setup_possible,
            admin_moderate_profiles,
            admin_moderate_images,
            admin_view_all_profiles,
            admin_view_private_info,
            admin_view_profile_history,
            admin_ban_profile,
            banned_edit_profile,
            view_public_profiles,
        );

        Ok(Account::new_from(state, capabilities))
    }

    pub async fn media_check_moderation_request_for_account(
        configuration: &Configuration,
        account_id: AccountIdLight,
    ) -> Result<(), Error<InternalGetCheckModerationRequestForAccountError>> {
        mediainternal_api::internal_get_check_moderation_request_for_account(
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
