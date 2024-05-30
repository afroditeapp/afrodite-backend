//! Handlers for internal from Server to Server state transfers and messages

use axum::extract::{Path, State};
use model::AccountId;
use server_api::app::ValidateModerationRequest;
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, GetConfig, GetInternalApi, ReadData},
    internal_api,
    utils::StatusCode,
};

pub const PATH_INTERNAL_GET_CHECK_MODERATION_REQUEST_FOR_ACCOUNT: &str =
    "/internal/media_api/moderation/request/:account_id";

/// Check that media server has correct state for completing initial setup.
///
#[utoipa::path(
    get,
    path = "/internal/media_api/moderation/request/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Successful."),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn internal_get_check_moderation_request_for_account<
    S: GetConfig + ReadData + GetAccounts + GetInternalApi + ValidateModerationRequest,
>(
    State(state): State<S>,
    Path(account_id): Path<AccountId>,
) -> Result<(), StatusCode> {
    MEDIA_INTERNAL
        .internal_get_check_moderation_request_for_account
        .incr();

    let account_id = state.get_internal_id(account_id).await?;

    if state.config().components().media {
        state.media_check_moderation_request_for_account(account_id).await?;
        Ok(())
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

create_counters!(
    MediaInternalCounters,
    MEDIA_INTERNAL,
    MEDIA_INTERNAL_COUNTERS_LIST,
    internal_get_check_moderation_request_for_account,
);
