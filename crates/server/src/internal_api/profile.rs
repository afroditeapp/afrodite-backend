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

pub async fn profile_initial_setup<S: GetAccessTokens + GetConfig + ReadData + WriteData + GetInternalApi>(
    state: &S,
    account_id: AccountIdInternal,
    profile_name: String,
) -> Result<(), InternalApiError> {
    if state.config().components().profile {
        state
            .write(move |cmds| async move {
                cmds.profile().profile_name(account_id, profile_name).await
            })
            .await
            .change_context(InternalApiError::DataError)
    } else {
        // TODO: Add method to internal profile API which will do the
        // initial setup (setting the name field) for user profile.
        Ok(())
    }
}

/// Profile visiblity is set first to the profile server and in addition
/// to changing the visibility the current proifle is returned (used for
/// changing visibility for media server).
pub async fn profile_api_set_profile_visiblity<S: GetAccessTokens + GetConfig + ReadData + WriteData + GetInternalApi>(
    state: &S,
    account_id: AccountIdInternal,
    boolean_setting: BooleanSetting,
) -> Result<(), InternalApiError> {
    if state.config().components().profile {
        state
            .write(move |data| async move {
                data.profile()
                    .profile_update_visibility(
                        account_id,
                        boolean_setting.value,
                        false, // False overrides updates
                    )
                    .await
            })
            .await
            .change_context(InternalApiError::DataError)?;

        let profile: ProfileInternal = state
            .read()
            .profile()
            .profile(account_id)
            .await
            .change_context(InternalApiError::DataError)?;

        // TODO: Remove?
        super::media::media_api_profile_visiblity(state, account_id, boolean_setting, profile.into())
            .await
            .change_context(InternalApiError::ApiRequest)?;

        Ok(())
    } else {
        // TODO: Request internal profile api
        todo!()
    }
}
