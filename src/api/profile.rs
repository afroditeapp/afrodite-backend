pub mod data;

use axum::{extract::Path, Json, TypedHeader};

use hyper::StatusCode;

use self::data::Profile;

use super::{model::AccountIdLight, utils::{get_account, get_account_from_api_key}, GetUsers};

use tracing::error;

use super::{utils::ApiKeyHeader, GetApiKeys, ReadDatabase, WriteDatabase};

// TODO: Add timeout for database commands

pub const PATH_GET_PROFILE: &str = "/profile/:account_id";

#[utoipa::path(
    get,
    path = "/profile/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "Get profile.", body = [Profile]),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_profile<S: ReadDatabase + GetUsers>(
    Path(id): Path<AccountIdLight>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {
    // TODO: Validate user id

    let account_id = get_account(&state, id, |account| account.id()).await?;

    state
        .read_database()
        .read_json::<Profile>(account_id)
        .await
        .map(|profile| profile.into())
        .map_err(|e| {
            error!("Get profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })
}

/// TODO: Remove this after benchmarking?
pub const PATH_GET_DEFAULT_PROFILE: &str = "/profile/default/:account_id";


/// TODO: Remove this at some point
#[utoipa::path(
    get,
    path = "/profile/default/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "Get default profile.", body = [Profile]),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_default_profile<S: ReadDatabase>(
    Path(_account_id): Path<AccountIdLight>,
    _state: S,
) -> Result<Json<Profile>, StatusCode> {
    let default = Profile::default();
    Ok(default.into())
}

pub const PATH_POST_PROFILE: &str = "/profile";

#[utoipa::path(
    post,
    path = "/profile",
    request_body = Profile,
    responses(
        (status = 200, description = "Update profile", body = [Profile]),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn post_profile<S: GetApiKeys + WriteDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Json(profile): Json<Profile>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = get_account_from_api_key(&state, api_key.key(), |account| account.id()).await?;

    state.write_database()
        .update_json(account_id, &profile)
        .await
        .map_err(|e| {
            error!("Post profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
        })?;

    Ok(())
}
