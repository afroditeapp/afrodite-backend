use axum::{extract::State, Extension};
use model::UpdateReportResult;
use model_chat::{AccountIdInternal, UpdateChatMessageReport};
use server_api::{create_open_api_router, S};
use server_data_chat::write::GetWriteCommandsChat;
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, WriteData},
    db_write,
    utils::{Json, StatusCode},
};

const PATH_POST_CHAT_MESSAGE_REPORT: &str = "/chat_api/chat_message_report";

/// Report chat message.
///
/// The report target must be a match.
#[utoipa::path(
    post,
    path = PATH_POST_CHAT_MESSAGE_REPORT,
    request_body = UpdateChatMessageReport,
    responses(
        (status = 200, description = "Successfull.", body = UpdateReportResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_chat_message_report(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateChatMessageReport>,
) -> Result<Json<UpdateReportResult>, StatusCode> {
    CHAT.post_chat_message_report.incr();

    let target = state.get_internal_id(update.target).await?;

    let result = db_write!(state, move |cmds| cmds
        .chat()
        .report()
        .report_chat_message(account_id, target, update.message))?;

    Ok(result.into())
}

create_open_api_router!(
        fn router_chat_report,
        post_chat_message_report,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_REPORT_COUNTERS_LIST,
    post_chat_message_report,
);
