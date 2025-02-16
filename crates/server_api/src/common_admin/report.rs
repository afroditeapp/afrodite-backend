use axum::{extract::State, Extension};
use model::{
    AccountIdInternal, GetReportList, Permissions, ReportDetailed,
};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use crate::{
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write_multiple, S,
};
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_WAITING_REPORT_PAGE: &str =
    "/common_api/admin/waiting_report_page";

#[utoipa::path(
    get,
    path = PATH_GET_WAITING_REPORT_PAGE,
    responses(
        (status = 200, description = "Successful", body = GetReportList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_waiting_report_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetReportList>, StatusCode> {
    COMMON.get_waiting_report_page.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .common_admin()
        .report()
        .get_waiting_report_list()
        .await?;

    Ok(r.into())
}

const PATH_POST_PROCESS_REPORT: &str = "/common_api/admin/process_report";

#[utoipa::path(
    post,
    path = PATH_POST_PROCESS_REPORT,
    request_body = ReportDetailed,
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
pub async fn post_process_report(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<ReportDetailed>,
) -> Result<(), StatusCode> {
    COMMON.post_process_report.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let creator = state.get_internal_id(data.info.creator).await?;
    let target = state.get_internal_id(data.info.target).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.common_admin()
            .report()
            .process_report(moderator_id, creator, target, data.info.report_type, data.content)
            .await?;
        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(
        fn router_report,
        get_waiting_report_page,
        post_process_report,
);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_ADMIN_REPORT_COUNTERS_LIST,
    get_waiting_report_page,
    post_process_report,
);
