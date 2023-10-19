use axum::{extract::Path, Extension};
use model::{AccountId, AccountIdInternal, Profile, SentLikesPage, ReceivedLikesPage, MatchesPage, SentBlocksPage, ReceivedBlocksPage, PendingMessagesPage, MessageNumber, UpdateMessageViewStatus, PendingMessageDeleteList, SendMessageToAccount};

use super::{utils::{Json, StatusCode}, GetAccessTokens, GetAccounts, GetInternalApi, ReadData, WriteData, db_write};

pub const PATH_POST_SEND_LIKE: &str = "/chat_api/send_like";

/// Send a like to some account.
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
pub async fn post_send_like<S: ReadData + GetAccounts + GetAccessTokens + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
    state: S,
) -> Result<(), StatusCode> {
    // TODO: Check is profile public and is age ok.

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    db_write!(state, move |cmds| {
        cmds.chat().like_profile(id, requested_profile)
    })?;

    Ok(())
}


pub const PATH_GET_SENT_LIKES: &str = "/chat_api/sent_likes";

/// Get sent likes.
///
/// Profile will not be returned if:
///
/// - Profile is hidden
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
pub async fn get_sent_likes<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<SentLikesPage>, StatusCode> {
    // TODO: Remove non public profiles?
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
pub async fn get_received_likes<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<ReceivedLikesPage>, StatusCode> {
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
pub async fn delete_like<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
    state: S,
) -> Result<(), StatusCode> {
    // TODO: Prevent deleting if the profile is a match

    Err(StatusCode::INTERNAL_SERVER_ERROR)
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
pub async fn get_matches<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<MatchesPage>, StatusCode> {
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
pub async fn post_block_profile<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
    state: S,
) -> Result<(), StatusCode> {
    Err(StatusCode::INTERNAL_SERVER_ERROR)
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
pub async fn post_unblock_profile<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
    state: S,
) -> Result<(), StatusCode> {
    // TODO: Delete only if profile is blocked

    Err(StatusCode::INTERNAL_SERVER_ERROR)
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
pub async fn get_sent_blocks<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<SentBlocksPage>, StatusCode> {
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
pub async fn get_received_blocks<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<ReceivedBlocksPage>, StatusCode> {
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
pub async fn get_pending_messages<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<PendingMessagesPage>, StatusCode> {

    Err(StatusCode::INTERNAL_SERVER_ERROR)
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
pub async fn delete_pending_messages<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    Json(list): Json<PendingMessageDeleteList>,
    state: S,
) -> Result<(), StatusCode> {

    Err(StatusCode::INTERNAL_SERVER_ERROR)
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
pub async fn get_message_number_of_latest_viewed_message<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
    state: S,
) -> Result<Json<MessageNumber>, StatusCode> {

    Err(StatusCode::INTERNAL_SERVER_ERROR)
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
pub async fn post_message_number_of_latest_viewed_message<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<UpdateMessageViewStatus>,
    state: S,
) -> Result<(), StatusCode> {

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub const PATH_POST_SEND_MESSAGE: &str =
    "/chat_api/send_message";

/// Send message
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
pub async fn post_send_message<S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData>(
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<SendMessageToAccount>,
    state: S,
) -> Result<(), StatusCode> {

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
