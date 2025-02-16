use axum::{extract::State, Extension};
use model_chat::{
    AccountIdInternal, GetChatReportList, Permissions, ProcessChatReport
};
use server_api::{
    app::{GetAccounts, WriteData},
    create_open_api_router, db_write_multiple, S,
};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_GET_CHAT_REPORT_PENDING_PROCESSING_LIST: &str =
    "/chat_api/admin/chat_report_pending_processing";

#[utoipa::path(
    get,
    path = PATH_GET_CHAT_REPORT_PENDING_PROCESSING_LIST,
    responses(
        (status = 200, description = "Successful", body = GetChatReportList),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_chat_report_pending_processing_list(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
) -> Result<Json<GetChatReportList>, StatusCode> {
    CHAT.get_chat_report_pending_processing_list.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = state
        .read()
        .chat_admin()
        .report()
        .get_report_list()
        .await?;

    Ok(r.into())
}

const PATH_POST_PROCESS_CHAT_REPORT: &str = "/chat_api/admin/process_chat_report";

#[utoipa::path(
    post,
    path = PATH_POST_PROCESS_CHAT_REPORT,
    request_body = ProcessChatReport,
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
pub async fn post_process_chat_report(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Extension(moderator_id): Extension<AccountIdInternal>,
    Json(data): Json<ProcessChatReport>,
) -> Result<(), StatusCode> {
    CHAT.post_process_chat_report.incr();

    if !permissions.admin_process_reports {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let creator = state.get_internal_id(data.creator).await?;
    let target = state.get_internal_id(data.target).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.chat_admin()
            .report()
            .process_report(moderator_id, creator, target, data.content)
            .await?;
        Ok(())
    })?;

    Ok(())
}

create_open_api_router!(
        fn router_admin_chat_report,
        get_chat_report_pending_processing_list,
        post_process_chat_report,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_ADMIN_CHAT_REPORT_COUNTERS_LIST,
    get_chat_report_pending_processing_list,
    post_process_chat_report,
);
