pub mod data;
pub mod internal;

use axum::{extract::Path, Json, TypedHeader};

use hyper::StatusCode;

use self::data::{
    Location, Profile, ProfileInternal, ProfilePage, ProfileUpdate, ProfileUpdateInternal,
};

use super::{model::AccountIdLight, GetInternalApi, GetUsers};

use tracing::error;

use super::{utils::ApiKeyHeader, GetApiKeys, ReadDatabase, WriteDatabase};

// TODO: Add timeout for database commands

pub const PATH_TODO: &str = "/chat_api/TODO/:account_id";

/// TODO
#[utoipa::path(
    get,
    path = "/chat_api/TODO/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "TODO", body = Profile),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Profile does not exist, is private or other server error.",
        ),
    ),
    security(("api_key" = [])),
)]
pub async fn get_todo<
    S: ReadDatabase + GetUsers + GetApiKeys + GetInternalApi + WriteDatabase,
>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Path(requested_profile): Path<AccountIdLight>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
