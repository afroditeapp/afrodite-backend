//! Handlers for internal from Server to Server state transfers and messages

use axum::extract::Path;
use hyper::StatusCode;
use model::{AccountId, BooleanSetting};
use tracing::error;

use super::{GetAccessTokens, GetConfig, WriteData};
use crate::api::{GetInternalApi, GetUsers, ReadData};

pub const PATH_INTERNAL_POST_UPDATE_PROFILE_VISIBLITY: &str =
    "/internal/profile_api/visibility/:account_id/:value";

#[utoipa::path(
    post,
    path = "/internal/profile_api/visiblity/{account_id}/{value}",
    params(AccountId, BooleanSetting),
    responses(
        (status = 200, description = "Visibility update successfull"),
        (status = 404, description = "No account found."),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn internal_post_update_profile_visibility<
    S: ReadData + GetUsers + GetInternalApi + GetAccessTokens + GetConfig + WriteData,
>(
    Path(account_id): Path<AccountId>,
    Path(value): Path<BooleanSetting>,
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

    state
        .internal_api()
        .profile_api_set_profile_visiblity(account_id, value)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
