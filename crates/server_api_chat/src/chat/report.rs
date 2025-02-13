use axum::{extract::{Query, State}, Extension};
use model::ReportQueryParams;
use model_chat::{AccountIdInternal, ChatReport, UpdateChatReport, UpdateChatReportResult};
use server_api::{create_open_api_router, S};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

const PATH_GET_CHAT_REPORT: &str = "/chat_api/chat_report";

/// Get chat report
#[utoipa::path(
    get,
    path = PATH_GET_CHAT_REPORT,
    params(ReportQueryParams),
    responses(
        (status = 200, description = "Successfull.", body = ChatReport),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_chat_report(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Query(report): Query<ReportQueryParams>,
) -> Result<Json<ChatReport>, StatusCode> {
    CHAT.get_chat_report.incr();

    let target = state.get_internal_id(report.target).await?;

    let report = state.read().chat().report().get_report(
        account_id,
        target,
    ).await?;

    Ok(report.into())
}

const PATH_POST_CHAT_REPORT: &str = "/chat_api/chat_report";

/// Update chat report.
///
/// The [ChatReportContent::is_against_video_calling] can be true only
/// when users are a match.
#[utoipa::path(
    post,
    path = PATH_POST_CHAT_REPORT,
    request_body = UpdateChatReport,
    responses(
        (status = 200, description = "Successfull.", body = UpdateChatReportResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_chat_report(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateChatReport>,
) -> Result<Json<UpdateChatReportResult>, StatusCode> {
    CHAT.post_chat_report.incr();

    let target = state.get_internal_id(update.target).await?;

    let result = db_write!(state, move |cmds| cmds
        .chat()
        .report()
        .update_report(account_id, target, update.content))?;

    Ok(result.into())
}

create_open_api_router!(
        fn router_chat_report,
        get_chat_report,
        post_chat_report,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_REPORT_COUNTERS_LIST,
    get_chat_report,
    post_chat_report,
);
