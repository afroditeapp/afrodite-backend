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

use super::InternalApiError;


/// Sync new Account to possible other servers.
/// Only account server can call this function.
pub async fn sync_account_state<S: GetConfig + GetInternalApi>(
    state: &S,
    account_id: AccountIdInternal,
    account: Account,
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
