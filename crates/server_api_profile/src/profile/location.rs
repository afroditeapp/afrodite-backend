use axum::{Extension, extract::State};
use model_profile::{AccountIdInternal, Location};
use server_api::{S, create_open_api_router, db_write};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;

use crate::{
    app::{ReadData, WriteData},
    utils::{Json, StatusCode},
};

const PATH_GET_LOCATION: &str = "/profile_api/location";

/// Get location for account which makes this request.
#[utoipa::path(
    get,
    path = PATH_GET_LOCATION,
    responses(
        (status = 200, description = "Get successfull.", body = Location),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_location(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<Location>, StatusCode> {
    PROFILE.get_location.incr();

    let location = state.read().profile().profile_location(account_id).await?;
    Ok(location.into())
}

const PATH_PUT_LOCATION: &str = "/profile_api/location";

/// Update location for account which makes this request.
#[utoipa::path(
    put,
    path = PATH_PUT_LOCATION,
    request_body = Location,
    responses(
        (status = 200, description = "Update successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_location(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(location): Json<Location>,
) -> Result<(), StatusCode> {
    PROFILE.put_location.incr();

    db_write!(state, move |cmds| cmds
        .profile()
        .profile_update_location(account_id, location)
        .await)
}

create_open_api_router!(fn router_location, get_location, put_location,);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_LOCATION_COUNTERS_LIST,
    get_location,
    put_location,
);
