use axum::{extract::State, Extension, Router};
use model::{AccountId, AccountIdInternal, AvailableProfileAttributes, FavoriteProfilesPage};
use simple_backend::create_counters;

use crate::{
    api::{
        db_write,
        utils::{Json, StatusCode},
    },
    app::{GetAccounts, GetConfig, ReadData, WriteData},
};

pub const PATH_GET_AVAILABLE_PROFILE_ATTRIBUTES: &str = "/profile_api/available_profile_attributes";

/// Get info what profile attributes server supports.
#[utoipa::path(
    get,
    path = "/profile_api/available_profile_attributes",
    responses(
        (status = 200, description = "Get successfull.", body = AvailableProfileAttributes),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_available_profile_attributes<S: GetConfig + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<AvailableProfileAttributes>, StatusCode> {
    PROFILE.get_available_profile_attributes.incr();
    let profile_state = state.read().profile().profile_state(account_id).await?;
    let info = AvailableProfileAttributes {
        info: state.config().profile_attributes().cloned(),
        sync_version: profile_state.profile_attributes_sync_version,
    };
    Ok(info.into())
}

pub fn attributes_router(s: crate::app::S) -> Router {
    use axum::routing::{delete, get, post};

    use crate::app::S;

    Router::new()
        .route(PATH_GET_AVAILABLE_PROFILE_ATTRIBUTES, get(get_available_profile_attributes::<S>))
        .with_state(s)
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ATTRIBUTES_COUNTERS_LIST,
    get_available_profile_attributes,
);
