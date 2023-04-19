//! Handlers for internal from Server to Server state transfers and messages

use axum::{
    extract::{BodyStream, Path},
    TypedHeader,
};
use headers::{ContentLength, ContentType};
use hyper::StatusCode;

use crate::api::{
    model::{AccountIdInternal, AccountIdLight},
    GetUsers, ReadDatabase,
};

use super::{super::account::data::AccountId, data::NewModerationRequest};

use super::data::ImageFileName;

pub const PATH_INTERNAL_GET_MODERATION_REQUEST_FOR_ACCOUNT: &str =
    "/internal/media_api/moderation/request/:account_id";

/// Check that current moderation request for account exists.
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
pub async fn internal_get_moderation_request_for_account<S: ReadDatabase + GetUsers>(
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

    let _request = state
        .read_database()
        .moderation_request(account_id)
        .await
        .map_err(|e| {
            tracing::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(())
}
