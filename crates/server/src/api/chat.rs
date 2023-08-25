use axum::Extension;
use axum::{extract::Path, TypedHeader};
use hyper::StatusCode;
use model::AccountIdInternal;
use tracing::error;

use super::{GetAccessTokens, GetAccounts};
use super::{GetInternalApi};
use super::{ReadData, utils::{Json}, WriteData};

use model::{
    Location, Profile, ProfileInternal, ProfilePage, ProfileUpdate, ProfileUpdateInternal, AccountId
};

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
pub async fn get_todo<
    S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData,
>(
    Extension(id): Extension<AccountIdInternal>,
    Path(requested_profile): Path<AccountId>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
