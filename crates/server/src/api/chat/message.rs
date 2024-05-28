use axum::{extract::State, Extension, Router};
use model::{
    AccountId, AccountIdInternal, EventToClientInternal, LatestViewedMessageChanged, MessageNumber,
    NotificationEvent, PendingMessageDeleteList, PendingMessagesPage, SendMessageToAccount,
    UpdateMessageViewStatus,
};
use simple_backend::create_counters;

use super::super::{
    db_write,
    utils::{Json, StatusCode},
};
use crate::{
    api::db_write_multiple,
    app::{GetAccounts, ReadData, WriteData},
};

pub const PATH_GET_PENDING_MESSAGES: &str = "/chat_api/pending_messages";

/// Get list of pending messages
#[utoipa::path(
    get,
    path = "/chat_api/pending_messages",
    responses(
        (status = 200, description = "Success.", body = PendingMessagesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_pending_messages<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<PendingMessagesPage>, StatusCode> {
    CHAT.get_pending_messages.incr();

    let page = state.read().chat().all_pending_messages(id).await?;
    Ok(page.into())
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

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;
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

    let message_sender = state
        .accounts()
        .get_internal_id(update_info.account_id_sender)
        .await?;
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

/// Send message to a match
#[utoipa::path(
    post,
    path = "/chat_api/send_message",
    request_body(content = SendMessageToAccount),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_send_message<S: GetAccounts + WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(message_info): Json<SendMessageToAccount>,
) -> Result<(), StatusCode> {
    CHAT.post_send_message.incr();

    let message_reciever = state
        .accounts()
        .get_internal_id(message_info.receiver)
        .await?;
    db_write_multiple!(state, move |cmds| {
        cmds.chat()
            .insert_pending_message_if_match(id, message_reciever, message_info.message)
            .await?;

        cmds.events()
            .send_notification(message_reciever, NotificationEvent::NewMessageReceived)
            .await?;

        Ok(())
    })?;

    Ok(())
}

pub fn message_router(s: crate::app::S) -> Router {
    use axum::routing::{delete, get, post};

    use crate::app::S;

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
