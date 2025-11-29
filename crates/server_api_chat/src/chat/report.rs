use axum::{Extension, extract::State};
use base64::Engine;
use model::{AdminNotificationTypes, UpdateReportResult};
use model_chat::{
    AccountIdInternal, NewChatMessageReportInternal, SignedMessageData, UpdateChatMessageReport,
};
use server_api::{
    S,
    app::{AdminNotificationProvider, DataSignerProvider},
    create_open_api_router, db_write,
};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;
use utils::encrypt::{decrypt_binary_message, verify_signed_binary_message};

use crate::{
    app::{GetAccounts, ReadData, WriteData},
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

    let signed_message = base64::engine::general_purpose::STANDARD
        .decode(update.backend_signed_message_base64)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let decryption_key = base64::engine::general_purpose::STANDARD
        .decode(update.decryption_key_base64)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data = state
        .data_signer()
        .verify_and_extract_backend_signed_data(&signed_message)
        .await?;
    let data = SignedMessageData::parse(&data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if data.receiver != update.target && data.sender != update.target {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let sender_account_id_internal = state.get_internal_id(data.sender).await?;
    let sender_public_key = state
        .read()
        .chat()
        .public_key()
        .get_public_key_data(sender_account_id_internal, data.sender_public_key_id)
        .await?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let signed_pgp_message = decrypt_binary_message(&data.message, &decryption_key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let client_message_bytes =
        verify_signed_binary_message(&signed_pgp_message, &sender_public_key)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let target = state.get_internal_id(update.target).await?;
    let report = NewChatMessageReportInternal {
        message_sender_account_id_uuid: data.sender,
        message_receiver_account_id_uuid: data.receiver,
        message_number: data.m,
        message_unix_time: data.unix_time,
        message_symmetric_key: decryption_key,
        client_message_bytes,
        backend_signed_message_bytes: signed_message,
    };

    let result = db_write!(state, move |cmds| cmds
        .chat()
        .report()
        .report_chat_message(account_id, target, report)
        .await)?;

    state
        .admin_notification()
        .send_notification_if_needed(AdminNotificationTypes::ProcessReports)
        .await;

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
