use axum::{Extension, extract::State};
use model::{
    AccountId, AccountIdInternal, EventToClientInternal, LatestViewedMessageChanged, MatchesPage,
    MessageNumber, NotificationEvent, PendingMessageDeleteList, PendingMessagesPage,
    ReceivedBlocksPage, ReceivedLikesPage, SendMessageToAccount, SentBlocksPage, SentLikesPage,
    UpdateMessageViewStatus,
};
use simple_backend::create_counters;

use super::{
    db_write,
    utils::{Json, StatusCode},
};
use crate::{app::{EventManagerProvider, GetAccounts, ReadData, WriteData}};

pub const PATH_POST_SEND_LIKE: &str = "/chat_api/send_like";

/// Send a like to some account. If both will like each other, then
/// the accounts will be a match.
#[utoipa::path(
    post,
    path = "/chat_api/send_like",
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_send_like<S: GetAccounts + WriteData + EventManagerProvider>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_send_like.incr();

    // TODO: Check is profile public and is age ok.

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    let new_state = db_write!(state, move |cmds| {
        cmds.chat().like_or_match_profile(id, requested_profile)
    })?;

    state
        .event_manager()
        .send_notification(requested_profile, model::NotificationEvent::LikesChanged)
        .await?;

    if new_state.is_match() {
        // State is now match so the account was removed from
        // received likes of the API caller.
        state
            .event_manager()
            .send_notification(id, model::NotificationEvent::LikesChanged)
            .await?;
    }

    Ok(())
}

pub const PATH_GET_SENT_LIKES: &str = "/chat_api/sent_likes";

/// Get sent likes.
///
/// Profile will not be returned if:
///
/// - Profile is hidden (not public)
/// - Profile is blocked
/// - Profile is a match
#[utoipa::path(
    get,
    path = "/chat_api/sent_likes",
    responses(
        (status = 200, description = "Success.", body = SentLikesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_sent_likes<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<SentLikesPage>, StatusCode> {
    CHAT.get_sent_likes.incr();

    let page = state.read().chat().all_sent_likes(id).await?;
    Ok(page.into())
}

pub const PATH_GET_RECEIVED_LIKES: &str = "/chat_api/received_likes";

/// Get received likes.
///
/// Profile will not be returned if:
/// - Profile is blocked
/// - Profile is a match
#[utoipa::path(
    get,
    path = "/chat_api/received_likes",
    responses(
        (status = 200, description = "Success.", body = ReceivedLikesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_received_likes<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<ReceivedLikesPage>, StatusCode> {
    CHAT.get_received_likes.incr();

    let page = state.read().chat().all_received_likes(id).await?;
    Ok(page.into())
}

pub const PATH_DELETE_LIKE: &str = "/chat_api/delete_like";

/// Delete sent like.
///
/// Delete will not work if profile is a match.
#[utoipa::path(
    delete,
    path = "/chat_api/delete_like",
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_like<S: GetAccounts + WriteData + EventManagerProvider>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.delete_like.incr();

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    db_write!(state, move |cmds| {
        cmds.chat().delete_like_or_block(id, requested_profile)
    })?;

    state
        .event_manager()
        .send_notification(requested_profile, model::NotificationEvent::LikesChanged)
        .await?;

    Ok(())
}

pub const PATH_GET_MATCHES: &str = "/chat_api/matches";

/// Get matches
#[utoipa::path(
    get,
    path = "/chat_api/matches",
    responses(
        (status = 200, description = "Success.", body = MatchesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_matches<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<MatchesPage>, StatusCode> {
    CHAT.get_matches.incr();

    let page = state.read().chat().all_matches(id).await?;
    Ok(page.into())
}

pub const PATH_POST_BLOCK_PROFILE: &str = "/chat_api/block_profile";

/// Block profile
#[utoipa::path(
    post,
    path = "/chat_api/block_profile",
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_block_profile<S: GetAccounts + WriteData + EventManagerProvider>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_block_profile.incr();

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    db_write!(state, move |cmds| {
        cmds.chat().block_profile(id, requested_profile)
    })?;

    state
        .event_manager()
        .send_notification(
            requested_profile,
            model::NotificationEvent::ReceivedBlocksChanged,
        )
        .await?;

    Ok(())
}

pub const PATH_POST_UNBLOCK_PROFILE: &str = "/chat_api/unblock_profile";

/// Unblock profile
#[utoipa::path(
    post,
    path = "/chat_api/unblock_profile",
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_unblock_profile<S: GetAccounts + WriteData + EventManagerProvider>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_unblock_profile.incr();

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    db_write!(state, move |cmds| {
        cmds.chat().delete_like_or_block(id, requested_profile)
    })?;

    state
        .event_manager()
        .send_notification(
            requested_profile,
            model::NotificationEvent::ReceivedBlocksChanged,
        )
        .await?;

    Ok(())
}

pub const PATH_GET_SENT_BLOCKS: &str = "/chat_api/sent_blocks";

/// Get list of sent blocks
#[utoipa::path(
    get,
    path = "/chat_api/sent_blocks",
    responses(
        (status = 200, description = "Success.", body = SentBlocksPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_sent_blocks<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<SentBlocksPage>, StatusCode> {
    CHAT.get_sent_blocks.incr();

    let page = state.read().chat().all_sent_blocks(id).await?;
    Ok(page.into())
}

// TODO: Add some block query info, so that server can send sync received blocks
//       list command to client.

pub const PATH_GET_RECEIVED_BLOCKS: &str = "/chat_api/received_blocks";

/// Get list of received blocks
#[utoipa::path(
    get,
    path = "/chat_api/received_blocks",
    responses(
        (status = 200, description = "Success.", body = ReceivedBlocksPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_received_blocks<S: ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<ReceivedBlocksPage>, StatusCode> {
    CHAT.get_received_blocks.incr();

    let page = state.read().chat().all_received_blocks(id).await?;
    Ok(page.into())
}

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
pub async fn post_message_number_of_latest_viewed_message<
    S: GetAccounts + WriteData + EventManagerProvider,
>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(update_info): Json<UpdateMessageViewStatus>,
) -> Result<(), StatusCode> {
    CHAT.post_message_number_of_latest_viewed_message.incr();

    let message_sender = state
        .accounts()
        .get_internal_id(update_info.account_id_sender)
        .await?;
    db_write!(state, move |cmds| {
        cmds.chat().update_message_number_of_latest_viewed_message(
            id,
            message_sender,
            update_info.message_number,
        )
    })?;

    state
        .event_manager()
        .send_connected_event(
            message_sender,
            EventToClientInternal::LatestViewedMessageChanged(LatestViewedMessageChanged {
                account_id_viewer: id.into(),
                new_latest_viewed_message: update_info.message_number,
            }),
        )
        .await?;
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
pub async fn post_send_message<S: GetAccounts + WriteData + EventManagerProvider>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(message_info): Json<SendMessageToAccount>,
) -> Result<(), StatusCode> {
    CHAT.post_send_message.incr();

    let message_reciever = state
        .accounts()
        .get_internal_id(message_info.receiver)
        .await?;
    db_write!(state, move |cmds| {
        cmds.chat()
            .insert_pending_message_if_match(id, message_reciever, message_info.message)
    })?;
    state
        .event_manager()
        .send_notification(message_reciever, NotificationEvent::NewMessageReceived)
        .await?;
    Ok(())
}


create_counters!(
    ChatCounters,
    CHAT,
    CHAT_COUNTERS_LIST,
    post_send_like,
    get_sent_likes,
    get_received_likes,
    delete_like,
    get_matches,
    post_block_profile,
    post_unblock_profile,
    get_sent_blocks,
    get_received_blocks,
    get_pending_messages,
    delete_pending_messages,
    get_message_number_of_latest_viewed_message,
    post_message_number_of_latest_viewed_message,
    post_send_message,
);
