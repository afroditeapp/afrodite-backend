use axum::{extract::State, Extension};
use model_media::{
    AccountIdInternal, GetMediaReportList, Permissions, ProcessMediaReport
};
use server_api::{
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write_multiple, S,
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_MEDIA_REPORT_PENDING_PROCESSING_LIST: &str =
    "/media_api/admin/media_report_pending_processing";

#[utoipa::path(
    get,
    path = PATH_GET_MEDIA_REPORT_PENDING_PROCESSING_LIST,
    responses(
        (status = 200, description = "Successful", body = GetMediaReportList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_media_report_pending_processing_list(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetMediaReportList>, StatusCode> {
    MEDIA.get_media_report_pending_processing_list.incr();

    if !permissions.admin_process_media_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .media_admin()
        .report()
        .get_report_list()
        .await?;

    Ok(r.into())
}

const PATH_POST_PROCESS_MEDIA_REPORT: &str = "/media_api/admin/process_media_report";

#[utoipa::path(
    post,
    path = PATH_POST_PROCESS_MEDIA_REPORT,
    request_body = ProcessMediaReport,
    responses(
        (status = 200, description = "Successful"),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn post_process_media_report(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<ProcessMediaReport>,
) -> Result<(), StatusCode> {
    MEDIA.post_process_media_report.incr();

    if !permissions.admin_process_media_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let creator = state.get_internal_id(data.creator).await?;
    let target = state.get_internal_id(data.target).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.media_admin()
            .report()
            .process_report(moderator_id, creator, target, data.profile_content)
            .await?;
        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(
        fn router_admin_media_report,
        get_media_report_pending_processing_list,
        post_process_media_report,
);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_ADMIN_MEDIA_REPORT,
    get_media_report_pending_processing_list,
    post_process_media_report,
);
