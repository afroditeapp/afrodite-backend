use axum::{body::Body, extract::{Query, State}, Extension, Router};
use axum_extra::TypedHeader;
use headers::ContentType;
use model::{
    AccountId, AccountIdInternal, EventToClientInternal, LatestViewedMessageChanged, MessageNumber, NotificationEvent, PendingMessageAcknowledgementList, SendMessageResult, SendMessageToAccountParams, SentMessageIdList, UpdateMessageViewStatus
};
use obfuscate_api_macro::obfuscate_api;
use server_data_chat::{read::GetReadChatCommands, write::{chat::PushNotificationAllowed, GetWriteCommandsChat}};
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

// TODO(prod): Chat improvements:
// - sign then encrypt - allows storing only signed messages and public
//   keys to message backup. The import tool will check the signatures
//   and prevent importing modified messages. The public keys must be
//   signed by server to make sure that those really are from the user.
//   Also the sign then encrypt will make reliable message reporting
//   possible as the messages are signed.
// - Server should store all public keys and max uploads for public keys
//   should be 1024.
// - Update pgp to new version and change keys to use X25519 and Ed25519.

#[obfuscate_api]
const PATH_GET_PENDING_MESSAGES: &str = "/chat_api/pending_messages";

/// Get list of pending messages.
///
/// The returned bytes is list of objects with following data:
/// - UTF-8 text length encoded as 16 bit little endian number.
/// - UTF-8 text which is PendingMessage JSON.
/// - Binary message data length as 16 bit little endian number.
/// - Binary message data
#[utoipa::path(
    get,
    path = PATH_GET_PENDING_MESSAGES,
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

#[obfuscate_api]
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
pub async fn post_add_receiver_acknowledgement<S: WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(list): Json<PendingMessageAcknowledgementList>,
) -> Result<(), StatusCode> {
    CHAT.delete_pending_messages.incr();

    db_write!(state, move |cmds| {
        cmds.chat()
            .add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(id, list.ids)
    })?;
    Ok(())
}

#[obfuscate_api]
const PATH_GET_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE: &str =
    "/chat_api/message_number_of_latest_viewed_message";

/// Get message number of the most recent message that the recipient has viewed.
#[utoipa::path(
    get,
    path = PATH_GET_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE,
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

#[obfuscate_api]
const PATH_POST_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE: &str =
    "/chat_api/message_number_of_latest_viewed_message";

/// Update message number of the most recent message that the recipient has viewed.
#[utoipa::path(
    post,
    path = PATH_POST_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE,
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

    let message_sender = state.get_internal_id(update_info.sender).await?;
    db_write_multiple!(state, move |cmds| {
        cmds.chat()
            .update_message_number_of_latest_viewed_message(
                id,
                message_sender,
                update_info.mn,
            )
            .await?;

        cmds.events()
            .send_connected_event(
                message_sender,
                EventToClientInternal::LatestViewedMessageChanged(LatestViewedMessageChanged {
                    viewer: id.into(),
                    new_latest_viewed_message: update_info.mn,
                }),
            )
            .await?;

        Ok(())
    })?;

    Ok(())
}

#[obfuscate_api]
const PATH_POST_SEND_MESSAGE: &str = "/chat_api/send_message";

/// Send message to a match.
///
/// Max pending message count is 50.
/// Max message size is u16::MAX.
///
/// The sender message ID must be value which server expects.
///
/// Sending will fail if one or two way block exists.
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
        let (result, push_notification_allowed) = cmds.chat()
            .insert_pending_message_if_match_and_not_blocked(
                id,
                message_reciever,
                bytes.into(),
                query_params.receiver_public_key_id,
                query_params.receiver_public_key_version,
                query_params.client_id,
                query_params.client_local_id,
            )
            .await?;

        if !result.is_err() {
            match push_notification_allowed {
                Some(PushNotificationAllowed) =>
                    cmds.events()
                        .send_notification(message_reciever, NotificationEvent::NewMessageReceived)
                        .await
                        .ignore_and_log_error(),
                None =>
                    cmds.events()
                        .send_connected_event(message_reciever, EventToClientInternal::NewMessageReceived)
                        .await
                        .ignore_and_log_error(),
            }
        }

        Ok(result)
    })?;

    Ok(result.into())
}

#[obfuscate_api]
const PATH_GET_SENT_MESSAGE_IDS: &str =
    "/chat_api/sent_message_ids";

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
pub async fn get_sent_message_ids<S: ReadData + GetAccounts>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<SentMessageIdList>, StatusCode> {
    CHAT.get_sent_message_ids.incr();
    let ids = state
        .read()
        .chat()
        .all_sent_messages(id)
        .await?;
    let id_list = SentMessageIdList {
        ids,
    };
    Ok(id_list.into())
}

#[obfuscate_api]
const PATH_POST_ADD_SENDER_ACKNOWLEDGEMENT: &str =
    "/chat_api/add_sender_acknowledgement";

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
pub async fn post_add_sender_acknowledgement<S: WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(id_list): Json<SentMessageIdList>,
) -> Result<(), StatusCode> {
    CHAT.post_add_sender_acknowledgement.incr();
    db_write!(state, move |cmds| {
        cmds.chat().add_sender_acknowledgement_and_delete_if_also_receiver_has_acknowledged(id, id_list.ids)
    })?;
    Ok(())
}

pub fn message_router<S: StateBase + GetAccounts + WriteData + ReadData>(s: S) -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route(PATH_GET_PENDING_MESSAGES_AXUM, get(get_pending_messages::<S>))
        .route(
            PATH_POST_ADD_RECEIVER_ACKNOWLEDGEMENT_AXUM,
            post(post_add_receiver_acknowledgement::<S>),
        )
        .route(
            PATH_GET_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE_AXUM,
            get(get_message_number_of_latest_viewed_message::<S>),
        )
        .route(
            PATH_POST_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE_AXUM,
            post(post_message_number_of_latest_viewed_message::<S>),
        )
        .route(PATH_POST_SEND_MESSAGE_AXUM, post(post_send_message::<S>))
        .route(PATH_GET_SENT_MESSAGE_IDS_AXUM, get(get_sent_message_ids::<S>))
        .route(PATH_POST_ADD_SENDER_ACKNOWLEDGEMENT_AXUM, post(post_add_sender_acknowledgement::<S>))
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
    get_sent_message_ids,
    post_add_sender_acknowledgement,
);
