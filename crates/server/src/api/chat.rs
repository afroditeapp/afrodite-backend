use axum::{extract::Path, Extension};
use hyper::StatusCode;
use model::{AccountId, AccountIdInternal, Profile};

use super::{utils::Json, GetAccessTokens, GetAccounts, GetInternalApi, ReadData, WriteData};

// TODO: Add timeout for database commands

pub const PATH_TODO: &str = "/chat_api/TODO/:account_id";

/// TODO
#[utoipa::path(
    get,
    path = "/chat_api/TODO/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "TODO", body = Profile),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Profile does not exist, is private or other server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_todo<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(_id): Extension<AccountIdInternal>,
    Path(_requested_profile): Path<AccountId>,
    _state: S,
) -> Result<Json<Profile>, StatusCode> {
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
