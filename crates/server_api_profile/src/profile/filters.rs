use axum::{extract::State, Extension};
use model_profile::{
    AccountIdInternal, GetProfileFilteringSettings, ProfileAttributeQuery, ProfileAttributeQueryResult, ProfileFilteringSettingsUpdate
};
use server_api::{create_open_api_router, S};
use server_data::DataError;
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use simple_backend_utils::IntoReportFromString;

use crate::{
    app::{GetConfig, ReadData, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

const PATH_POST_GET_QUERY_AVAILABLE_PROFILE_ATTRIBUTES: &str = "/profile_api/query_available_profile_attributes";

/// Query profile attributes using attribute ID list.
///
/// The HTTP method is POST because HTTP GET does not allow request body.
#[utoipa::path(
    post,
    path = PATH_POST_GET_QUERY_AVAILABLE_PROFILE_ATTRIBUTES,
    request_body = ProfileAttributeQuery,
    responses(
        (status = 200, description = "Successfull.", body = ProfileAttributeQueryResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_query_available_profile_attributes(
    State(state): State<S>,
    Json(query): Json<ProfileAttributeQuery>,
) -> Result<Json<ProfileAttributeQueryResult>, StatusCode> {
    PROFILE.post_get_query_available_profile_attributes.incr();
    let info = ProfileAttributeQueryResult {
        values: state.config().profile_attributes().map(|a| a.query_attributes(query.values)).unwrap_or_default(),
    };
    Ok(info.into())
}

const PATH_GET_PROFILE_FILTERING_SETTINGS: &str = "/profile_api/profile_filtering_settings";

/// Get current profile filtering settings.
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_FILTERING_SETTINGS,
    responses(
        (status = 200, description = "Successfull.", body = GetProfileFilteringSettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_filtering_settings(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<GetProfileFilteringSettings>, StatusCode> {
    PROFILE.get_profile_attribute_filters.incr();
    let filters = state
        .read()
        .profile()
        .profile_filtering_settings(account_id)
        .await?;
    Ok(filters.into())
}

const PATH_POST_PROFILE_FILTERING_SETTINGS: &str = "/profile_api/profile_filtering_settings";

/// Set profile filtering settings.
#[utoipa::path(
    post,
    path = PATH_POST_PROFILE_FILTERING_SETTINGS,
    request_body = ProfileFilteringSettingsUpdate,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_profile_filtering_settings(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(data): Json<ProfileFilteringSettingsUpdate>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_filtering_settings.incr();
    let validated = data
        .validate(state.config().profile_attributes())
        .into_error_string(DataError::NotAllowed)?;
    db_write!(state, move |cmds| cmds
        .profile()
        .update_profile_filtering_settings(account_id, validated))
}

create_open_api_router!(
        fn router_filters,
        post_get_query_available_profile_attributes,
        get_profile_filtering_settings,
        post_profile_filtering_settings,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_FILTERS_COUNTERS_LIST,
    post_get_query_available_profile_attributes,
    get_profile_attribute_filters,
    post_profile_filtering_settings,
);
