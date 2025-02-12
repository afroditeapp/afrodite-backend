use axum::{extract::{Query, State}, Extension};
use model::{ReportQueryParams, UpdateReportResult};
use model_media::{AccountIdInternal, MediaReport, UpdateMediaReport};
use server_api::{create_open_api_router, S};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

const PATH_GET_MEDIA_REPORT: &str = "/media_api/media_report";

/// Get media report
#[utoipa::path(
    get,
    path = PATH_GET_MEDIA_REPORT,
    params(ReportQueryParams),
    responses(
        (status = 200, description = "Successfull.", body = MediaReport),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_media_report(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Query(report): Query<ReportQueryParams>,
) -> Result<Json<MediaReport>, StatusCode> {
    MEDIA.get_media_report.incr();

    let target = state.get_internal_id(report.target).await?;

    let report = state.read().media().report().get_report(
        account_id,
        target,
    ).await?;

    Ok(report.into())
}

const PATH_POST_MEDIA_REPORT: &str = "/media_api/media_report";

/// Update media report.
///
/// If profile content is reported and it is bot moderated, the content's
/// moderation state changes to
/// [model_media::ContentModerationState::WaitingHumanModeration].
#[utoipa::path(
    post,
    path = PATH_POST_MEDIA_REPORT,
    request_body = UpdateMediaReport,
    responses(
        (status = 200, description = "Successfull.", body = UpdateReportResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_media_report(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateMediaReport>,
) -> Result<Json<UpdateReportResult>, StatusCode> {
    MEDIA.post_media_report.incr();

    let target = state.get_internal_id(update.target).await?;

    let result = db_write!(state, move |cmds| cmds
        .media()
        .report()
        .update_report(account_id, target, update.content))?;

    Ok(result.into())
}

create_open_api_router!(
        fn router_media_report,
        get_media_report,
        post_media_report,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_REPORT_MEDIA_REPORT_COUNTERS_LIST,
    get_media_report,
    post_media_report,
);
