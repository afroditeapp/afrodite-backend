use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, AvailableProfileAttributes, ProfileAttributeFilterList, ProfileAttributeFilterListUpdate};
use simple_backend::create_counters;
use simple_backend_utils::IntoReportFromString;

use crate::{
    api::{
        db_write,
        utils::{Json, StatusCode},
    },
    app::{GetConfig, ReadData, WriteData}, data::DataError,
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

pub const PATH_GET_PROFILE_ATTRIBUTE_FILTERS: &str = "/profile_api/profile_attribute_filters";

/// Get current profile attribute filter values.
#[utoipa::path(
    get,
    path = "/profile_api/profile_attribute_filters",
    responses(
        (status = 200, description = "Successfull.", body = ProfileAttributeFilterList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_attribute_filters<S: ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ProfileAttributeFilterList>, StatusCode> {
    PROFILE.get_profile_attribute_filters.incr();
    let filters = state.read().profile().profile_attribute_filters(account_id).await?;
    Ok(filters.into())
}

pub const PATH_POST_PROFILE_ATTRIBUTE_FILTERS: &str = "/profile_api/profile_attribute_filters";

/// Set profile attribute filter values.
#[utoipa::path(
    post,
    path = "/profile_api/profile_attribute_filters",
    request_body = ProfileAttributeFilterListUpdate,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_profile_attribute_filters<S: WriteData + GetConfig>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(data): Json<ProfileAttributeFilterListUpdate>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_attributes_filters.incr();
    let validated = data.validate(state.config().profile_attributes())
        .into_error_string(DataError::NotAllowed)?;
    db_write!(state, move |cmds| cmds.profile().update_profile_attribute_filters(account_id, validated))
}

pub fn attributes_router(s: crate::app::S) -> Router {
    use axum::routing::{get, post};

    use crate::app::S;

    Router::new()
        .route(PATH_GET_AVAILABLE_PROFILE_ATTRIBUTES, get(get_available_profile_attributes::<S>))
        .route(PATH_GET_PROFILE_ATTRIBUTE_FILTERS, get(get_profile_attribute_filters::<S>))
        .route(PATH_POST_PROFILE_ATTRIBUTE_FILTERS, post(post_profile_attribute_filters::<S>))
        .with_state(s)
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ATTRIBUTES_COUNTERS_LIST,
    get_available_profile_attributes,
    get_profile_attribute_filters,
    post_profile_attributes_filters,
);
