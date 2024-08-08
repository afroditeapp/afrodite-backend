use axum::{body::Body, extract::{Query, State}, Extension, Router};
use axum_extra::TypedHeader;
use headers::ContentType;
use model::{
    AccountId, AccountIdInternal, EventToClientInternal, LatestViewedMessageChanged, MessageNumber, NotificationEvent, PendingMessageDeleteList, SendMessageResult, SendMessageToAccountParams, UpdateMessageViewStatus
};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;
use tracing::error;

use super::super::{
    db_write,
    utils::{Json, StatusCode},
};
use crate::{
    app::{GetAccounts, ReadData, StateBase, WriteData},
    db_write_multiple,
};

pub const PATH_GET_PENDING_MESSAGES: &str = "/chat_api/pending_messages";

/// Get list of pending messages.
///
/// The returned bytes is list of objects with following data:
/// - UTF-8 text length encoded as 16 bit little endian number.
/// - UTF-8 text which is PendingMessage JSON.
/// - Binary message data length as 16 bit little endian number.
/// - Binary message data
#[utoipa::path(
    get,
    path = "/chat_api/pending_messages",
    responses(
        (status = 200, description = "Success.", body = Vec<u8>, content_type = "application/octet-stream"),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_pending_messages<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<(TypedHeader<ContentType>, Vec<u8>), StatusCode> {
    CHAT.get_pending_messages.incr();
    let pending_messages = state.read().chat().all_pending_messages(id).await?;

    let mut bytes: Vec<u8> = vec![];
    for p in pending_messages {
        let pending_message_json = match serde_json::to_string(&p.pending_message) {
            Ok(s) => s,
            Err(_) => {
                error!("Deserializing pending message failed");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let json_length: u16 = match pending_message_json
            .len()
            .try_into() {
                Ok(len) => len,
                Err(_) => {
                    error!("Pending message JSON is too large");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };
        bytes.extend_from_slice(&json_length.to_le_bytes());
        bytes.extend_from_slice(pending_message_json.as_bytes());
        let message_length: u16 = match p.message
            .len()
            .try_into() {
                Ok(len) => len,
                Err(_) => {
                    error!("Pending message data is too large");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };
        bytes.extend_from_slice(&message_length.to_le_bytes());
        bytes.extend_from_slice(&p.message);
    }

    Ok((TypedHeader(ContentType::octet_stream()), bytes))
}

pub const PATH_DELETE_PENDING_MESSAGES: &str = "/chat_api/pending_messages";

/// Delete list of pending messages
#[utoipa::path(
    delete,
    path = "/chat_api/pending_messages",
    request_body(content = PendingMessageDeleteList),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_pending_messages<S: WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(list): Json<PendingMessageDeleteList>,
) -> Result<(), StatusCode> {
    CHAT.delete_pending_messages.incr();

    db_write!(state, move |cmds| {
        cmds.chat()
            .delete_pending_message_list(id, list.messages_ids)
    })?;
    Ok(())
}

pub const PATH_GET_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE: &str =
    "/chat_api/message_number_of_latest_viewed_message";

/// Get message number of the most recent message that the recipient has viewed.
#[utoipa::path(
    get,
    path = "/chat_api/message_number_of_latest_viewed_message",
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success.", body = MessageNumber),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_message_number_of_latest_viewed_message<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<Json<MessageNumber>, StatusCode> {
    CHAT.get_message_number_of_latest_viewed_message.incr();

    let requested_profile = state.get_internal_id(requested_profile).await?;
    let number = state
        .read()
        .chat()
        .message_number_of_latest_viewed_message(id, requested_profile)
        .await?;
    Ok(number.into())
}

pub const PATH_POST_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE: &str =
    "/chat_api/message_number_of_latest_viewed_message";

/// Update message number of the most recent message that the recipient has viewed.
#[utoipa::path(
    post,
    path = "/chat_api/message_number_of_latest_viewed_message",
    request_body(content = UpdateMessageViewStatus),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_message_number_of_latest_viewed_message<S: GetAccounts + WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(update_info): Json<UpdateMessageViewStatus>,
) -> Result<(), StatusCode> {
    CHAT.post_message_number_of_latest_viewed_message.incr();

    let message_sender = state.get_internal_id(update_info.account_id_sender).await?;
    db_write_multiple!(state, move |cmds| {
        cmds.chat()
            .update_message_number_of_latest_viewed_message(
                id,
                message_sender,
                update_info.message_number,
            )
            .await?;

        cmds.events()
            .send_connected_event(
                message_sender,
                EventToClientInternal::LatestViewedMessageChanged(LatestViewedMessageChanged {
                    account_id_viewer: id.into(),
                    new_latest_viewed_message: update_info.message_number,
                }),
            )
            .await?;

        Ok(())
    })?;

    Ok(())
}

pub const PATH_POST_SEND_MESSAGE: &str = "/chat_api/send_message";

/// Send message to a match.
///
/// Max pending message count is 50.
/// Max message size is u16::MAX.
#[utoipa::path(
    post,
    path = PATH_POST_SEND_MESSAGE,
    params(SendMessageToAccountParams),
    request_body(content = Vec<u8>, content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "Success.", body = SendMessageResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error or message data related error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_send_message<S: GetAccounts + WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Query(query_params): Query<SendMessageToAccountParams>,
    message_bytes: Body,
) -> Result<Json<SendMessageResult>, StatusCode> {
    CHAT.post_send_message.incr();

    let bytes = axum::body::to_bytes(message_bytes, u16::MAX.into())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let message_reciever = state.get_internal_id(query_params.receiver).await?;
    let result = db_write_multiple!(state, move |cmds| {
        let result = cmds.chat()
            .insert_pending_message_if_match(
                id,
                message_reciever,
                bytes.into(),
                query_params.receiver_public_key_id,
                query_params.receiver_public_key_version,
            )
            .await?;

        cmds.events()
            .send_notification(message_reciever, NotificationEvent::NewMessageReceived)
            .await?;

        Ok(result)
    })?;

    Ok(result.into())
}

pub fn message_router<S: StateBase + GetAccounts + WriteData + ReadData>(s: S) -> Router {
    use axum::routing::{delete, get, post};

    Router::new()
        .route(PATH_GET_PENDING_MESSAGES, get(get_pending_messages::<S>))
        .route(
            PATH_DELETE_PENDING_MESSAGES,
            delete(delete_pending_messages::<S>),
        )
        .route(
            PATH_GET_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE,
            get(get_message_number_of_latest_viewed_message::<S>),
        )
        .route(
            PATH_POST_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE,
            post(post_message_number_of_latest_viewed_message::<S>),
        )
        .route(PATH_POST_SEND_MESSAGE, post(post_send_message::<S>))
        .with_state(s)
}

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_MESSAGE_COUNTERS_LIST,
    get_pending_messages,
    delete_pending_messages,
    get_message_number_of_latest_viewed_message,
    post_message_number_of_latest_viewed_message,
    post_send_message,
);
