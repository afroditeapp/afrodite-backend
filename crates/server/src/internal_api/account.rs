use api_internal::{Configuration, InternalApi};
use config::{Config, InternalApiUrls};
use hyper::StatusCode;
use model::{
    AccessToken, Account, AccountIdInternal, AccountState, BooleanSetting, Capabilities, Profile,
    ProfileInternal,
};
use tracing::{error, info, warn};

use crate::{data::{read::ReadCommands, utils::AccessTokenManager}};
use crate::{
    app::{GetAccessTokens, GetConfig, GetInternalApi, ReadData, WriteData},
    data::WrappedWithInfo,
    result::{Result, WrappedContextExt, WrappedResultExt},
};

use super::{common::AuthResponse, InternalApiError};

/// TODO: Perhaps change getting account state every time from the shared state
///       so assume that it is account server's responsibility to keeping the
///       ohter servers up to date.
pub async fn get_account_state<S: GetAccessTokens + GetConfig + ReadData + GetInternalApi>(
    state: &S,
    account_id: AccountIdInternal,
) -> Result<Account, InternalApiError> {
    if state.config().components().account {
        state.read()
            .account()
            .account(account_id)
            .await
            .change_context(InternalApiError::DataError)
    } else {
        let account =
            InternalApi::get_account_state(state.internal_api_client().account()?, account_id.as_id())
                .await
                .change_context(InternalApiError::ApiRequest)?;

        Ok(account)
    }
}

pub struct Data<'a> {
    pub capabilities: &'a mut Capabilities,
    pub state: &'a mut AccountState,
    pub is_profile_public: &'a mut bool,
}


/// Only account server can modify the state. Does nothing if the server
/// does not have account component enabled.
///
/// Returns the modified capabilities.
pub async fn modify_and_sync_account_state<S: GetAccessTokens + GetConfig + ReadData + WriteData + GetInternalApi>(
    state: &S,
    account_id: AccountIdInternal,
    action: impl FnOnce(Data),
) -> Result<Capabilities, InternalApiError> {
    if !state.config().components().account {
        warn!("Account component not enabled, cannot modify account state");
        // TODO: Would it be better to return error here?
        return Err(InternalApiError::MissingComponent.report());
    }

    let mut current = state
        .read()
        .account()
        .account(account_id)
        .await
        .change_context(InternalApiError::DataError)?
        .into_capablities();

    let mut shared_state = state
        .read()
        .common()
        .shared_state(account_id)
        .await
        .change_context(InternalApiError::DataError)?;

    action(Data {
        capabilities: &mut current,
        state: &mut shared_state.account_state,
        is_profile_public: &mut shared_state.is_profile_public,
    });

    let modified_capabilities_copy = current.clone();
    let modified_shared_state_copy = shared_state.clone();
    state
        .write(move |cmds| async move {
            cmds.account()
                .update_account_state_and_capabilities(
                    account_id,
                    Some(modified_shared_state_copy),
                    Some(modified_capabilities_copy),
                )
                .await
        })
        .await
        .change_context(InternalApiError::DataError)?;

    // TODO add sync account state command to common internal api

    if !state.config().components().profile {
        // let account =
        // InternalApi::get_account_state(self.api_client.account()?, account_id.as_id())
        //     .await
        //     .change_context(InternalApiError::ApiRequest)?;
    }

    if !state.config().components().media {
        // let account =
        // InternalApi::get_account_state(self.api_client.account()?, account_id.as_id())
        //     .await
        //     .change_context(InternalApiError::ApiRequest)?;
    }

    if !state.config().components().chat {
        // let account =
        // InternalApi::get_account_state(self.api_client.account()?, account_id.as_id())
        //     .await
        //     .change_context(InternalApiError::ApiRequest)?;
    }

    Ok(current)
}
