use api_internal::InternalApi;
use model::AccountIdInternal;
use server_common::internal_api::InternalApiError;

use crate::{
    app::{GetConfig, GetInternalApi},
    result::{Result, WrappedResultExt},
    S,
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
pub async fn media_check_moderation_request_for_account(
    state: &S,
    account_id: AccountIdInternal,
) -> Result<(), InternalApiError> {
    if state.config().components().media {
        state
            .data_all_access()
            .check_moderation_request_for_account(account_id)
            .await
            .change_context(InternalApiError::DataError)
    } else {
        InternalApi::media_check_moderation_request_for_account(
            state.internal_api_client().media()?,
            account_id.as_id(),
        )
        .await
        .change_context(InternalApiError::MissingValue)
    }
}
