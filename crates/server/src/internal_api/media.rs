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

use super::{account, InternalApiError};

/// Check that media server has correct state for completing initial setup.
///
/// Requirements:
///  - Account must have a moderation request.
///  - The current or pending security image of the account is in the request.
///  - The current or pending first profile image of the account is in the
///    request.
///
/// TODO(prod): Make sure that moderation request is not removed when admin
///             interacts with it.
pub async fn media_check_moderation_request_for_account<S: GetAccessTokens + GetConfig + ReadData + GetInternalApi>(
    state: &S,
    account_id: AccountIdInternal,
) -> Result<(), InternalApiError> {
    if state.config().components().media {
        let request = state
            .read()
            .moderation_request(account_id)
            .await
            .change_context_with_info(InternalApiError::DataError, account_id)?
            .ok_or(InternalApiError::MissingValue)
            .with_info(account_id)?;

        let account_media =
            state.read().media().current_account_media(account_id).await.change_context_with_info(InternalApiError::DataError, account_id)?;

        // Check security content
        let current_or_pending_security_content = account_media.security_content_id
            .or(account_media.pending_security_content_id);
        if let Some(content) = current_or_pending_security_content {
            if !content.secure_capture {
                return Err(InternalApiError::SecureCaptureFlagFalse).with_info(account_id)
            }
            if request.content.exists(content.content_id) {
                return Err(InternalApiError::SecurityContentNotInModerationRequest).with_info(account_id)
            }
        } else {
            return Err(InternalApiError::SecurityContentNotSet).with_info(account_id)
        }

        // Check first profile content
        let current_or_pending_profile_content = account_media.profile_content_id_0
            .or(account_media.pending_profile_content_id_0);
        if let Some(content) = current_or_pending_profile_content {
            if request.content.exists(content.content_id) {
                return Err(InternalApiError::ContentNotInModerationRequest).with_info(account_id)
            }
        } else {
            return Err(InternalApiError::ContentNotSet).with_info(account_id)
        }

        Ok(())
    } else {
        InternalApi::media_check_moderation_request_for_account(
            state.internal_api_client().media()?,
            account_id.as_id(),
        )
        .await
        .change_context(InternalApiError::MissingValue)
    }
}

// TODO: Can media_api_profile_visiblity be removed?

pub async fn media_api_profile_visiblity<S: GetConfig>(
    state: &S,
    _account_id: AccountIdInternal,
    _boolean_setting: BooleanSetting,
    _current_profile: Profile,
) -> Result<(), InternalApiError> {
    if state.config().components().media {
        // TODO: Save visibility information to cache?
        Ok(())
    } else {
        // TODO: request to internal media API
        Ok(())
    }
}

// TODO: Prevent creating a new moderation request when there is camera
// image in the current one. Or also make possible to change the ongoing
// moderation request but leave the camera image. Where information about
// the camera image should be stored?
