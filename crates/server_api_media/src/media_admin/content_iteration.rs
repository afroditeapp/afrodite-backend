use axum::{
    Extension,
    extract::{Query, State},
};
use model::{Permissions, UnixTime};
use model_media::{GetMediaContentIterationPageParams, MediaContentIterationPage};
use server_api::{S, create_open_api_router};
use server_data_media::read::GetReadMediaCommands;
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_MEDIA_CONTENT_ITERATION_PAGE: &str = "/media_api/get_media_content_iteration_page";

/// Get a page of media content items iterated in descending content upload
/// time order. Max 25 values are returned.
///
/// `only_security_content` can be used to iterate only security content
/// items. Default is to return all content items (profile and security
/// content).
///
/// # Access
/// * Permission [Permissions::admin_moderate_media_content]
#[utoipa::path(
    get,
    path = PATH_GET_MEDIA_CONTENT_ITERATION_PAGE,
    params(GetMediaContentIterationPageParams),
    responses(
        (status = 200, description = "Successful.", body = MediaContentIterationPage),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_media_content_iteration_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetMediaContentIterationPageParams>,
) -> Result<Json<MediaContentIterationPage>, StatusCode> {
    MEDIA_ADMIN.get_media_content_iteration_page.incr();

    let only_security_content = params.only_security_content.unwrap_or(false);

    if !permissions.admin_moderate_media_content {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .media_admin()
        .media_content_iteration_page(
            only_security_content,
            params.page,
            UnixTime::new(params.origin_unix_time),
        )
        .await?;

    Ok(r.into())
}

create_open_api_router!(
        fn router_admin_content_iteration,
        get_media_content_iteration_page,
);

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_CONTENT_ITERATION_COUNTERS_LIST,
    get_media_content_iteration_page,
);
