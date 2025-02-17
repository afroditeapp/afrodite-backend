use axum::{extract::{Query, State}, Extension};
use model::{
    AccountIdInternal, GetReportList, Permissions, ProcessReport, ReportIteratorQuery, ReportIteratorQueryInternal, UnixTime
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
    request_body = ProcessReport,
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
    Json(data): Json<ProcessReport>,
) -> Result<(), StatusCode> {
    COMMON.post_process_report.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let creator = state.get_internal_id(data.creator).await?;
    let target = state.get_internal_id(data.target).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.common_admin()
            .report()
            .process_report(moderator_id, creator, target, data.report_type, data.content)
            .await?;
        Ok(())
    })?;

    Ok(())
}

const PATH_GET_LATEST_REPORT_ITERATOR_START_POSITION: &str =
    "/common_api/admin/latest_report_iterator_start_position";

#[utoipa::path(
    get,
    path = PATH_GET_LATEST_REPORT_ITERATOR_START_POSITION,
    responses(
        (status = 200, description = "Successful", body = UnixTime),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_latest_report_iterator_start_position(
    State(_state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<UnixTime>, StatusCode> {
    COMMON.get_latest_report_iterator_start_position.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let previous_time = UnixTime::current_time().decrement();
    Ok(previous_time.into())
}

const PATH_GET_REPORT_ITERATOR_PAGE: &str =
    "/common_api/admin/report_iterator_page";

#[utoipa::path(
    get,
    path = PATH_GET_REPORT_ITERATOR_PAGE,
    params(ReportIteratorQuery),
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
pub async fn get_report_iterator_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Query(query): Query<ReportIteratorQuery>,
) -> Result<Json<GetReportList>, StatusCode> {
    COMMON.get_report_iterator_page.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let query_account = state.get_internal_id(query.aid).await?;

    let query_internal = ReportIteratorQueryInternal {
        start_position: query.start_position,
        page: query.page,
        aid: query_account,
        mode: query.mode,
    };

    let r = state
        .read()
        .common_admin()
        .report()
        .get_report_iterator_page(query_internal)
        .await?;

    Ok(r.into())
}

create_open_api_router!(
        fn router_report,
        get_waiting_report_page,
        post_process_report,
        get_latest_report_iterator_start_position,
        get_report_iterator_page,
);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_ADMIN_REPORT_COUNTERS_LIST,
    get_waiting_report_page,
    post_process_report,
    get_latest_report_iterator_start_position,
    get_report_iterator_page,
);
