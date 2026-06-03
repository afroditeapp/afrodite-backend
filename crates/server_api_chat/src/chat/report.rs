use axum::{Extension, extract::State};
use base64::Engine;
use model::{AdminNotificationTypes, UpdateReportResult};
use model_chat::{
    AccountIdInternal, NewChatMessageReportInternal, SignedMessageData, UpdateChatMessageReports,
};
use server_api::{
    S,
    app::{AdminNotificationProvider, ApiLimitsProvider, DataSignerProvider},
    create_open_api_router, db_write,
};
use server_data_chat::write::GetWriteCommandsChat;
use simple_backend::create_counters;
use utils::encrypt::{decrypt_binary_message, unwrap_signed_binary_message};

use crate::{
    app::{GetAccounts, WriteData},
    utils::{Json, StatusCode},
};

const PATH_POST_CHAT_MESSAGE_REPORTS: &str = "/chat_api/chat_message_reports";
const MAX_REPORTED_MESSAGES_PER_REQUEST: usize = 10;

/// Report chat message.
///
/// The report target must be a match.
/// Supports reporting at most 10 messages per request.
#[utoipa::path(
    post,
    path = PATH_POST_CHAT_MESSAGE_REPORTS,
    request_body = UpdateChatMessageReports,
    responses(
        (status = 200, description = "Successfull.", body = UpdateReportResult),
        (status = 400, description = "Invalid request."),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_chat_message_reports(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(update): Json<UpdateChatMessageReports>,
) -> Result<Json<UpdateReportResult>, StatusCode> {
    CHAT.post_chat_message_reports.incr();
    state.api_limits(account_id).common().send_report().await?;

    if update.messages.is_empty() || update.messages.len() > MAX_REPORTED_MESSAGES_PER_REQUEST {
        return Err(StatusCode::BAD_REQUEST);
    }

    let target = state.get_internal_id(update.target).await?;
    let mut reports = Vec::with_capacity(update.messages.len());

    for report_message in update.messages {
        let signed_message = base64::engine::general_purpose::STANDARD
            .decode(report_message.server_signed_message_base64)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let decryption_key = base64::engine::general_purpose::STANDARD
            .decode(report_message.decryption_key_base64)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let data = state
            .data_signer()
            .verify_and_extract_backend_signed_data(&signed_message)
            .await?;
        let data =
            SignedMessageData::parse(&data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if data.recipient != target.as_id() && data.sender != target.as_id() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let signed_pgp_message = decrypt_binary_message(&data.message, &decryption_key)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let client_message_bytes = unwrap_signed_binary_message(&signed_pgp_message)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let report = NewChatMessageReportInternal {
            message_sender_account_id_uuid: data.sender,
            message_recipient_account_id_uuid: data.recipient,
            message_number: data.message_number,
            message_unix_time: data.unix_time,
            message_symmetric_key: decryption_key,
            client_message_bytes,
            server_signed_message_bytes: signed_message,
        };

        reports.push(report);
    }

    let result = db_write!(state, move |cmds| cmds
        .chat()
        .report()
        .report_chat_message(account_id, target, reports)
        .await)?;

    state
        .admin_notification()
        .send_notification_if_needed(AdminNotificationTypes::ProcessReports)
        .await;

    Ok(result.into())
}

create_open_api_router!(
        fn router_chat_report,
        post_chat_message_reports,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_REPORT_COUNTERS_LIST,
    post_chat_message_reports,
);
