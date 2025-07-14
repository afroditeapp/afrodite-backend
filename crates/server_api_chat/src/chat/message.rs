use axum::{
    Extension,
    body::Body,
    extract::{Path, Query, State},
};
use axum_extra::TypedHeader;
use headers::ContentType;
use model::{GetConversationId, NotificationEvent};
use model_chat::{
    AccountId, AccountIdInternal, EventToClientInternal, GetSentMessage,
    PendingMessageAcknowledgementList, SendMessageResult, SendMessageToAccountParams,
    SentMessageId, SentMessageIdList, add_minimal_i64,
};
use server_api::{
    S,
    app::{ApiUsageTrackerProvider, DataSignerProvider},
    create_open_api_router,
};
use server_data_chat::{
    read::GetReadChatCommands,
    write::{GetWriteCommandsChat, chat::PushNotificationAllowed},
};
use simple_backend::create_counters;
use tracing::error;

use super::super::utils::{Json, StatusCode};
use crate::{
    app::{GetAccounts, ReadData, WriteData},
    db_write_multiple,
};

const PATH_GET_PENDING_MESSAGES: &str = "/chat_api/pending_messages";

/// Get list of pending messages.
///
/// The returned bytes is list of objects with following data:
/// - Binary data length as minimal i64
/// - Binary data
///
/// Minimal i64 has this format:
/// - i64 byte count (u8, values: 1, 2, 4, 8)
/// - i64 bytes (little-endian)
///
/// Binary data is binary PGP message which contains backend signed
/// binary data. The binary data contains:
/// - Version (u8, values: 1)
/// - Sender AccountId UUID big-endian bytes (16 bytes)
/// - Receiver AccountId UUID big-endian bytes (16 bytes)
/// - Sender public key ID (minimal i64)
/// - Receiver public key ID (minimal i64)
/// - Message ID (minimal i64)
/// - Unix time (minimal i64)
/// - Message data
#[utoipa::path(
    get,
    path = PATH_GET_PENDING_MESSAGES,
    responses(
        (status = 200, description = "Success.", body = inline(model::BinaryData), content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_pending_messages(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    CHAT.get_pending_messages.incr();
    let pending_messages = state.read().chat().all_pending_messages(id).await?;

    let mut bytes: Vec<u8> = vec![];
    for m in pending_messages {
        let message_length: i64 = match m.len().try_into() {
            Ok(len) => len,
            Err(_) => {
                error!("Pending message data is too large");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        add_minimal_i64(&mut bytes, message_length);
        bytes.extend_from_slice(&m);
    }

    Ok((TypedHeader(ContentType::octet_stream()), bytes))
}

const PATH_POST_ADD_RECEIVER_ACKNOWLEDGEMENT: &str = "/chat_api/add_receiver_acknowledgement";

#[utoipa::path(
    post,
    path = PATH_POST_ADD_RECEIVER_ACKNOWLEDGEMENT,
    request_body(content = PendingMessageAcknowledgementList),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_add_receiver_acknowledgement(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(list): Json<PendingMessageAcknowledgementList>,
) -> Result<(), StatusCode> {
    CHAT.delete_pending_messages.incr();

    db_write_multiple!(state, move |cmds| {
        cmds.chat()
            .add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(id, list.ids)
            .await
    })?;
    Ok(())
}

const PATH_POST_SEND_MESSAGE: &str = "/chat_api/send_message";

/// Send message to a match.
///
/// Max pending message count is 50.
/// Max message size is u16::MAX.
///
/// The sender message ID must be value which server expects.
///
/// Sending will fail if one or two way block exists.
///
/// Only the latest public key for sender and receiver can be used when
/// sending a message.
#[utoipa::path(
    post,
    path = PATH_POST_SEND_MESSAGE,
    params(SendMessageToAccountParams),
    request_body(content = inline(model::BinaryData), content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Success.", body = SendMessageResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error or message data related error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_send_message(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Query(query_params): Query<SendMessageToAccountParams>,
    message_bytes: Body,
) -> Result<Json<SendMessageResult>, StatusCode> {
    CHAT.post_send_message.incr();
    state
        .api_usage_tracker()
        .incr(id, |u| &u.post_send_message)
        .await;

    let bytes = axum::body::to_bytes(message_bytes, u16::MAX.into())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(message_reciever) = state.get_internal_id_optional(query_params.receiver).await else {
        return Ok(SendMessageResult::receiver_blocked_sender_or_receiver_not_found().into());
    };
    let keys = state.data_signer().keys().await?;
    let result = db_write_multiple!(state, move |cmds| {
        let (result, push_notification_allowed) = cmds
            .chat()
            .insert_pending_message_if_match_and_not_blocked(
                id,
                message_reciever,
                bytes.into(),
                query_params.sender_public_key_id,
                query_params.receiver_public_key_id,
                query_params.client_id,
                query_params.client_local_id,
                keys,
            )
            .await?;

        if !result.is_err() {
            match push_notification_allowed {
                Some(PushNotificationAllowed) => cmds
                    .events()
                    .send_notification(message_reciever, NotificationEvent::NewMessageReceived)
                    .await
                    .ignore_and_log_error(),
                None => cmds
                    .events()
                    .send_connected_event(
                        message_reciever,
                        EventToClientInternal::NewMessageReceived,
                    )
                    .await
                    .ignore_and_log_error(),
            }
        }

        Ok(result)
    })?;

    Ok(result.into())
}

const PATH_POST_GET_SENT_MESSAGE: &str = "/chat_api/sent_message";

/// Receive unreceived [model_chat::SignedMessageData]
/// for sent message.
///
/// This is HTTP POST route only to allow JSON request body.
#[utoipa::path(
    post,
    path = PATH_POST_GET_SENT_MESSAGE,
    request_body = SentMessageId,
    responses(
        (status = 200, description = "Success.", body = GetSentMessage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_sent_message(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(message): Json<SentMessageId>,
) -> Result<Json<GetSentMessage>, StatusCode> {
    CHAT.post_get_sent_message.incr();
    let data = state.read().chat().get_sent_message(id, message).await?;
    Ok(data.into())
}

const PATH_GET_SENT_MESSAGE_IDS: &str = "/chat_api/sent_message_ids";

#[utoipa::path(
    get,
    path = PATH_GET_SENT_MESSAGE_IDS,
    responses(
        (status = 200, description = "Success.", body = SentMessageIdList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_sent_message_ids(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<SentMessageIdList>, StatusCode> {
    CHAT.get_sent_message_ids.incr();
    let ids = state.read().chat().all_sent_messages(id).await?;
    let id_list = SentMessageIdList { ids };
    Ok(id_list.into())
}

const PATH_POST_ADD_SENDER_ACKNOWLEDGEMENT: &str = "/chat_api/add_sender_acknowledgement";

#[utoipa::path(
    post,
    path = PATH_POST_ADD_SENDER_ACKNOWLEDGEMENT,
    request_body(content = SentMessageIdList),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_add_sender_acknowledgement(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(id_list): Json<SentMessageIdList>,
) -> Result<(), StatusCode> {
    CHAT.post_add_sender_acknowledgement.incr();
    db_write_multiple!(state, move |cmds| {
        cmds.chat()
            .add_sender_acknowledgement_and_delete_if_also_receiver_has_acknowledged(
                id,
                id_list.ids,
            )
            .await
    })?;
    Ok(())
}

const PATH_GET_CONVERSATION_ID: &str = "/chat_api/conversation_id/{aid}";

/// Get account specific conversation ID which can be used to display
/// new message received notifications.
///
/// The ID is available only for accounts which are a match.
#[utoipa::path(
    get,
    path = PATH_GET_CONVERSATION_ID,
    params(AccountId),
    responses(
        (status = 200, description = "Success.", body = GetConversationId),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_conversation_id(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Path(requested): Path<AccountId>,
) -> Result<Json<GetConversationId>, StatusCode> {
    CHAT.get_conversation_id.incr();

    let requested = state.get_internal_id(requested).await?;
    let Some(interaction) = state
        .read()
        .chat()
        .account_interaction(id, requested)
        .await?
    else {
        return Ok(GetConversationId::default().into());
    };

    let value = GetConversationId {
        value: interaction.conversation_id_for_account(requested),
    };

    Ok(value.into())
}

create_open_api_router!(
        fn router_message,
        get_pending_messages,
        post_add_receiver_acknowledgement,
        post_send_message,
        post_get_sent_message,
        get_sent_message_ids,
        post_add_sender_acknowledgement,
        get_conversation_id,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_MESSAGE_COUNTERS_LIST,
    get_pending_messages,
    delete_pending_messages,
    post_send_message,
    post_get_sent_message,
    get_sent_message_ids,
    post_add_sender_acknowledgement,
    get_conversation_id,
);
