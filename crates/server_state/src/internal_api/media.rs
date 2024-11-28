use api_internal::InternalApi;
use model::AccountIdInternal;
use server_common::{data::WrappedWithInfo, internal_api::InternalApiError};
use server_data_media::read::GetReadMediaCommands;

use crate::{
    app::{GetConfig, GetInternalApi, ReadData},
    result::{Result, WrappedResultExt},
};

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
pub async fn media_check_moderation_request_for_account<
    S: GetConfig + ReadData + GetInternalApi,
>(
    state: &S,
    account_id: AccountIdInternal,
) -> Result<(), InternalApiError> {
    if state.config().components().media {
        let request = state
            .read()
            .media()
            .moderation_request(account_id)
            .await
            .change_context_with_info(InternalApiError::DataError, account_id)?
            .ok_or(InternalApiError::MissingValue)
            .with_info(account_id)?;

        let account_media = state
            .read()
            .media()
            .current_account_media(account_id)
            .await
            .change_context_with_info(InternalApiError::DataError, account_id)?;

        // Check security content
        let current_or_pending_security_content = account_media
            .security_content_id
            .or(account_media.pending_security_content_id);
        if let Some(content) = current_or_pending_security_content {
            if !content.secure_capture {
                return Err(InternalApiError::SecureCaptureFlagFalse).with_info(account_id);
            }
            if request.content.find(content.content_id()).is_none() {
                return Err(InternalApiError::SecurityContentNotInModerationRequest)
                    .with_info(account_id);
            }
        } else {
            return Err(InternalApiError::SecurityContentNotSet).with_info(account_id);
        }

        // Check first profile content
        let current_or_pending_profile_content = account_media
            .profile_content_id_0
            .or(account_media.pending_profile_content_id_0);
        if let Some(content) = current_or_pending_profile_content {
            if request.content.find(content.content_id()).is_none() {
                return Err(InternalApiError::ContentNotInModerationRequest).with_info(account_id);
            }
        } else {
            return Err(InternalApiError::ContentNotSet).with_info(account_id);
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
