use model::{Account, AccountIdInternal};
use tracing::warn;

use super::InternalApiError;
use crate::{
    app::GetConfig,
    result::{Result, WrappedContextExt}, S,
};

/// Sync new Account to possible other servers.
/// Only account server can call this function.
pub async fn sync_account_state(
    state: &S,
    _account_id: AccountIdInternal,
    _account: Account,
) -> Result<(), InternalApiError> {
    if !state.config().components().account {
        warn!("Account component not enabled, cannot send new Account to other servers");
        return Err(InternalApiError::MissingComponent.report());
    }

    // TODO(microservice): Add sync account state command to common internal api

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

    Ok(())
}

pub async fn sync_unlimited_likes(
    _state: &S,
    _account_id: AccountIdInternal,
) -> Result<(), InternalApiError> {
    // TODO(microservice): sync unlimited likes
    Ok(())
}

pub async fn sync_birthdate(
    _state: &S,
    _account_id: AccountIdInternal,
) -> Result<(), InternalApiError> {
    // TODO(microservice): birthdate
    Ok(())
}
