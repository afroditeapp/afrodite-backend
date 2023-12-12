//! Handlers for internal from Server to Server state transfers and messages

use axum::extract::Path;
use model::{AccountId, BooleanSetting, Profile};

use crate::{
    api::utils::{Json, StatusCode},
    app::{GetAccounts, GetConfig, GetInternalApi, ReadData},
};

pub const PATH_INTERNAL_GET_CHECK_MODERATION_REQUEST_FOR_ACCOUNT: &str =
    "/internal/media_api/moderation/request/:account_id";

/// Check that current moderation request for account exists. Requires also
/// that request contains camera image.
///
#[utoipa::path(
    get,
    path = "/internal/media_api/moderation/request/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Get moderation request was successfull."),
        (status = 404, description = "No account or moderation request found."),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn internal_get_check_moderation_request_for_account<S: ReadData + GetAccounts>(
    Path(account_id): Path<AccountId>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state.accounts().get_internal_id(account_id).await?;

    let request = state
        .read()
        .moderation_request(account_id)
        .await?
        .ok_or(StatusCode::NOT_FOUND)?;

    if request.content.slot_1_is_security_image() {
        Ok(())
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub const PATH_INTERNAL_POST_UPDATE_PROFILE_IMAGE_VISIBLITY: &str =
    "/internal/media_api/visibility/:account_id/:value";

#[utoipa::path(
    post,
    path = "/internal/media_api/visiblity/{account_id}/{value}",
    params(AccountId, BooleanSetting),
    request_body(content = Profile),
    responses(
        (status = 200, description = "Visibility update successfull"),
        (status = 404, description = "No account found."),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn internal_post_update_profile_image_visibility<
    S: ReadData + GetAccounts + GetInternalApi + GetConfig,
>(
    Path(account_id): Path<AccountId>,
    Path(value): Path<BooleanSetting>,
    Json(profile): Json<Profile>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state.accounts().get_internal_id(account_id).await?;

    state
        .internal_api()
        .media_api_profile_visiblity(account_id, value, profile)
        .await?;
    Ok(())
}
