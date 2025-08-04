use axum::{Extension, extract::State};
use model_profile::{
    AccountIdInternal, GetProfileFilters, ProfileAttributesConfigQuery,
    ProfileAttributesConfigQueryResult, ProfileFiltersUpdate,
};
use server_api::{S, create_open_api_router, db_write};
use server_data::DataError;
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use simple_backend_utils::IntoReportFromString;

use crate::{
    app::{GetConfig, ReadData, WriteData},
    utils::{Json, StatusCode},
};

const PATH_POST_GET_QUERY_PROFILE_ATTRIBUTES_CONFIG: &str =
    "/profile_api/query_profile_attributes_config";

/// Query profile attributes from profile attributes config
/// using profile attribute ID list.
///
/// The HTTP method is POST because HTTP GET does not allow request body.
#[utoipa::path(
    post,
    path = PATH_POST_GET_QUERY_PROFILE_ATTRIBUTES_CONFIG,
    request_body = ProfileAttributesConfigQuery,
    responses(
        (status = 200, description = "Successfull.", body = ProfileAttributesConfigQueryResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_query_profile_attributes_config(
    State(state): State<S>,
    Json(query): Json<ProfileAttributesConfigQuery>,
) -> Result<Json<ProfileAttributesConfigQueryResult>, StatusCode> {
    PROFILE.post_get_query_profile_attributes_config.incr();
    let info = ProfileAttributesConfigQueryResult {
        values: state
            .config()
            .profile_attributes()
            .map(|a| a.query_attributes(query.values))
            .unwrap_or_default(),
    };
    Ok(info.into())
}

const PATH_GET_PROFILE_FILTERS: &str = "/profile_api/profile_filters";

/// Get current profile filters.
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_FILTERS,
    responses(
        (status = 200, description = "Successfull.", body = GetProfileFilters),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_filters(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<GetProfileFilters>, StatusCode> {
    PROFILE.get_profile_filters.incr();
    let filters = state.read().profile().profile_filters(account_id).await?;
    Ok(filters.into())
}

const PATH_POST_PROFILE_FILTERS: &str = "/profile_api/profile_filters";

/// Set profile filters.
#[utoipa::path(
    post,
    path = PATH_POST_PROFILE_FILTERS,
    request_body = ProfileFiltersUpdate,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_profile_filters(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(data): Json<ProfileFiltersUpdate>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_filters.incr();
    let validated = data
        .validate(state.config().profile_attributes())
        .into_error_string(DataError::NotAllowed)?;
    db_write!(state, move |cmds| cmds
        .profile()
        .update_profile_filters(account_id, validated)
        .await)
}

create_open_api_router!(
        fn router_filters,
        post_get_query_profile_attributes_config,
        get_profile_filters,
        post_profile_filters,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_FILTERS_COUNTERS_LIST,
    post_get_query_profile_attributes_config,
    get_profile_filters,
    post_profile_filters,
);
