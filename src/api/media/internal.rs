//! Handlers for internal from Server to Server state transfers and messages

use axum::{extract::Path, Json};

use hyper::StatusCode;

use crate::api::{model::{AccountIdLight, Profile, BooleanSetting}, GetUsers, ReadDatabase, GetInternalApi};

use tracing::{error, info};


pub const PATH_INTERNAL_GET_CHECK_MODERATION_REQUEST_FOR_ACCOUNT: &str =
    "/internal/media_api/moderation/request/:account_id";

/// Check that current moderation request for account exists. Requires also
/// that request contains camera image.
///
#[utoipa::path(
    get,
    path = "/internal/media_api/moderation/request/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "Get moderation request was successfull."),
        (status = 404, description = "No account or moderation request found."),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn internal_get_check_moderation_request_for_account<S: ReadDatabase + GetUsers>(
    Path(account_id): Path<AccountIdLight>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state
        .users()
        .get_internal_id(account_id)
        .await
        .map_err(|e| {
            tracing::error!("{}", e);
            StatusCode::NOT_FOUND
        })?;

    let request = state
        .read_database()
        .moderation_request(account_id)
        .await
        .map_err(|e| {
            tracing::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if request.content.camera() {
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
    params(AccountIdLight, BooleanSetting),
    request_body(content = Profile),
    responses(
        (status = 200, description = "Visibility update successfull"),
        (status = 404, description = "No account found."),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn internal_post_update_profile_image_visibility<S: ReadDatabase + GetUsers + GetInternalApi>(
    Path(account_id): Path<AccountIdLight>,
    Path(value): Path<BooleanSetting>,
    Json(profile): Json<Profile>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state
        .users()
        .get_internal_id(account_id)
        .await
        .map_err(|e| {
            tracing::error!("{}", e);
            StatusCode::NOT_FOUND
        })?;

    state.internal_api().media_api_profile_visiblity(account_id, value, profile).await.map_err(|e| {
        error!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
