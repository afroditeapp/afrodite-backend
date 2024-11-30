use axum::{extract::State, Extension};
use model_profile::{
    AccountIdInternal, AvailableProfileAttributes, ProfileAttributeFilterList,
    ProfileAttributeFilterListUpdate,
};
use obfuscate_api_macro::obfuscate_api;
use server_api::{create_open_api_router, S};
use server_data::DataError;
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use simple_backend_utils::IntoReportFromString;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{GetConfig, ReadData, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_GET_AVAILABLE_PROFILE_ATTRIBUTES: &str = "/profile_api/available_profile_attributes";

/// Get info what profile attributes server supports.
#[utoipa::path(
    get,
    path = PATH_GET_AVAILABLE_PROFILE_ATTRIBUTES,
    responses(
        (status = 200, description = "Get successfull.", body = AvailableProfileAttributes),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_available_profile_attributes(
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

#[obfuscate_api]
const PATH_GET_PROFILE_ATTRIBUTE_FILTERS: &str = "/profile_api/profile_attribute_filters";

/// Get current profile attribute filter values.
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_ATTRIBUTE_FILTERS,
    responses(
        (status = 200, description = "Successfull.", body = ProfileAttributeFilterList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_attribute_filters(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ProfileAttributeFilterList>, StatusCode> {
    PROFILE.get_profile_attribute_filters.incr();
    let filters = state
        .read()
        .profile()
        .profile_attribute_filters(account_id)
        .await?;
    Ok(filters.into())
}

#[obfuscate_api]
const PATH_POST_PROFILE_ATTRIBUTE_FILTERS: &str = "/profile_api/profile_attribute_filters";

/// Set profile attribute filter values.
#[utoipa::path(
    post,
    path = PATH_POST_PROFILE_ATTRIBUTE_FILTERS,
    request_body = ProfileAttributeFilterListUpdate,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_profile_attribute_filters(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(data): Json<ProfileAttributeFilterListUpdate>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_attributes_filters.incr();
    let validated = data
        .validate(state.config().profile_attributes())
        .into_error_string(DataError::NotAllowed)?;
    db_write!(state, move |cmds| cmds
        .profile()
        .update_profile_attribute_filters(account_id, validated))
}

pub fn attributes_router(s: S) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_available_profile_attributes,
        get_profile_attribute_filters,
        post_profile_attribute_filters,
    )
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ATTRIBUTES_COUNTERS_LIST,
    get_available_profile_attributes,
    get_profile_attribute_filters,
    post_profile_attributes_filters,
);
