use axum::{
    Extension,
    extract::{Query, State},
};
use model::{Permissions, UnixTime};
use model_media::{
    GetMediaContentIterationPageParams, MediaContentFiltersInternal, MediaContentIdDbValue,
    MediaContentIterationPage, MediaContentIteratorPage, MediaContentIteratorSettings,
    MediaContentIteratorSettingsInternal,
};
use server_api::{S, app::GetAccounts, create_open_api_router};
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

const PATH_GET_LATEST_MEDIA_CONTENT_ID_DB: &str = "/media_api/get_latest_media_content_id_db";

/// Get latest created media content ID DB
///
/// # Access
/// - Permission [model::Permissions::admin_moderate_media_content]
#[utoipa::path(
    get,
    path = PATH_GET_LATEST_MEDIA_CONTENT_ID_DB,
    responses(
        (status = 200, description = "Successful.", body = MediaContentIdDbValue),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_latest_media_content_id_db(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<MediaContentIdDbValue>, StatusCode> {
    MEDIA_ADMIN.get_latest_media_content_id_db.incr();

    if !permissions.admin_moderate_media_content {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .media_admin()
        .get_latest_media_content_id_db()
        .await?;

    Ok(r.into())
}

const PATH_GET_ADMIN_MEDIA_CONTENT_ITERATOR_PAGE: &str =
    "/media_api/get_admin_media_content_iterator_page";

/// Get admin media content iterator page
///
/// HTTP method is POST to allow JSON request body for filters and extra
/// info settings.
///
/// # Access
/// - Permission [model::Permissions::admin_moderate_media_content]
#[utoipa::path(
    post,
    path = PATH_GET_ADMIN_MEDIA_CONTENT_ITERATOR_PAGE,
    request_body = MediaContentIteratorSettings,
    responses(
        (status = 200, description = "Successful.", body = MediaContentIteratorPage),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Internal server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_admin_media_content_iterator_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(settings): Json<MediaContentIteratorSettings>,
) -> Result<Json<MediaContentIteratorPage>, StatusCode> {
    MEDIA_ADMIN.get_admin_media_content_iterator_page.incr();

    if !permissions.admin_moderate_media_content {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let filters = settings.filters();

    let moderator_account_id = match filters.moderator_account_id {
        Some(id) => Some(state.get_internal_id(id).await?.into_db_id()),
        None => None,
    };

    let moderator_account_id_not = match filters.moderator_account_id_not {
        Some(id) => Some(state.get_internal_id(id).await?.into_db_id()),
        None => None,
    };

    let settings_internal = MediaContentIteratorSettingsInternal {
        start_position: settings.start_position,
        page: settings.page,
        filters: MediaContentFiltersInternal {
            moderation_states: filters.moderation_states,
            initial_content: filters.initial_content,
            secure_capture: filters.secure_capture,
            in_use: filters.in_use,
            uploaded_after_unix_time: filters.uploaded_after_unix_time,
            uploaded_before_unix_time: filters.uploaded_before_unix_time,
            face_detected: filters.face_detected,
            face_detected_manual_set: filters.face_detected_manual_set,
            moderator_account_id,
            moderator_account_id_not,
        },
        extra_info: settings.extra_info(),
    };

    let r = state
        .read()
        .media_admin()
        .media_content_iterator_page(settings_internal)
        .await?;

    Ok(Json(r))
}

create_open_api_router!(
        fn router_admin_content_iteration,
        get_media_content_iteration_page,
        get_latest_media_content_id_db,
        get_admin_media_content_iterator_page,
);

create_counters!(
    MediaAdminCounters,
    MEDIA_ADMIN,
    MEDIA_ADMIN_CONTENT_ITERATION_COUNTERS_LIST,
    get_media_content_iteration_page,
    get_latest_media_content_id_db,
    get_admin_media_content_iterator_page,
);
