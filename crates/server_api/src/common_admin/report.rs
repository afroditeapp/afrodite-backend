use axum::{
    Extension,
    extract::{Query, State},
};
use model::{
    AccountIdInternal, GetChatMessageReports, GetChatMessageReportsInternal, GetReportList,
    Permissions, ProcessReports, ReportIteratorQuery, ReportIteratorQueryInternal, UnixTime,
    WaitingReportPageQuery,
};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use simple_backend::create_counters;

use crate::{
    S,
    app::{GetAccounts, ReadData, WriteData},
    create_open_api_router, db_write,
    utils::{Json, StatusCode},
};

const PATH_GET_WAITING_REPORT_PAGE: &str = "/common_api/waiting_report_page";

#[utoipa::path(
    get,
    path = PATH_GET_WAITING_REPORT_PAGE,
    params(WaitingReportPageQuery),
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
    Query(query): Query<WaitingReportPageQuery>,
) -> Result<Json<GetReportList>, StatusCode> {
    COMMON.get_waiting_report_page.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .common_admin()
        .report()
        .get_waiting_report_list(&query.wanted_report_types)
        .await?;

    Ok(r.into())
}

const PATH_POST_PROCESS_REPORTS: &str = "/common_api/process_reports";

#[utoipa::path(
    post,
    path = PATH_POST_PROCESS_REPORTS,
    request_body = ProcessReports,
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
pub async fn post_process_reports(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<ProcessReports>,
) -> Result<(), StatusCode> {
    COMMON.post_process_reports.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    db_write!(state, move |cmds| {
        cmds.common_admin()
            .report()
            .process_reports(moderator_id, data.values)
            .await?;
        Ok(())
    })?;

    Ok(())
}

const PATH_GET_LATEST_REPORT_ITERATOR_START_POSITION: &str =
    "/common_api/latest_report_iterator_start_position";

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

const PATH_POST_GET_REPORT_ITERATOR_PAGE: &str = "/common_api/report_iterator_page";

/// Get report iterator page.
///
/// The HTTP method is POST because HTTP GET does not allow request body.
#[utoipa::path(
    post,
    path = PATH_POST_GET_REPORT_ITERATOR_PAGE,
    request_body = ReportIteratorQuery,
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
pub async fn post_get_report_iterator_page(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(query): Json<ReportIteratorQuery>,
) -> Result<Json<GetReportList>, StatusCode> {
    COMMON.post_get_report_iterator_page.incr();

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

const PATH_POST_GET_CHAT_MESSAGE_REPORTS: &str = "/chat_api/get_chat_message_reports";

/// Get all chat message reports. The reports are ordered by message
/// sending order from oldest to latest.
#[utoipa::path(
    post,
    path = PATH_POST_GET_CHAT_MESSAGE_REPORTS,
    request_body = GetChatMessageReports,
    responses(
        (status = 200, description = "Successfull.", body = GetReportList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_chat_message_reports(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Json(settings): Json<GetChatMessageReports>,
) -> Result<Json<GetReportList>, StatusCode> {
    COMMON.post_get_chat_message_reports.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let creator = state.get_internal_id(settings.creator).await?;
    let target = state.get_internal_id(settings.target).await?;

    let query_internal = GetChatMessageReportsInternal {
        creator,
        target,
        only_not_processed: settings.only_not_processed,
    };

    let r = state
        .read()
        .common_admin()
        .report()
        .get_chat_message_reports(query_internal)
        .await?;

    Ok(r.into())
}

create_open_api_router!(
        fn router_report,
        get_waiting_report_page,
        post_process_reports,
        get_latest_report_iterator_start_position,
        post_get_report_iterator_page,
        post_get_chat_message_reports,
);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_ADMIN_REPORT_COUNTERS_LIST,
    get_waiting_report_page,
    post_process_reports,
    get_latest_report_iterator_start_position,
    post_get_report_iterator_page,
    post_get_chat_message_reports,
);
