use axum::{
    Extension,
    extract::{Query, State},
};
use model::UnixTime;
use model_profile::{
    GetProfileStringIterationPageParams, Permissions, ProfileStringIterationPage,
    ProfileStringModerationContentType,
};
use server_api::{S, create_open_api_router};
use server_data_profile::read::GetReadProfileCommands;
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_PROFILE_STRING_ITERATION_PAGE: &str =
    "/profile_api/get_profile_string_iteration_page";

/// Get a page of profile string values iterated in descending profile edited
/// time order. Max 25 values are returned.
///
/// Profiles without the string value are not included.
///
/// # Access
/// * [Permissions::admin_moderate_profile_names] or
///   [Permissions::admin_moderate_profile_texts] depending
///   on [GetProfileStringIterationPageParams::content_type].
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_STRING_ITERATION_PAGE,
    params(GetProfileStringIterationPageParams),
    responses(
        (status = 200, description = "Successful.", body = ProfileStringIterationPage),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_string_iteration_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetProfileStringIterationPageParams>,
) -> Result<Json<ProfileStringIterationPage>, StatusCode> {
    PROFILE.get_profile_string_iteration_page.incr();

    match params.content_type {
        ProfileStringModerationContentType::ProfileName => {
            if !permissions.admin_moderate_profile_names {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        ProfileStringModerationContentType::ProfileText => {
            if !permissions.admin_moderate_profile_texts {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    let r = state
        .read()
        .profile_admin()
        .iterator()
        .profile_string_iteration_page(
            params.content_type,
            params.page,
            UnixTime::new(params.origin_unix_time),
        )
        .await?;

    Ok(r.into())
}

create_open_api_router!(
        fn router_admin_profile_string_iteration,
        get_profile_string_iteration_page,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_PROFILE_STRING_ITERATION_COUNTERS_LIST,
    get_profile_string_iteration_page,
);
