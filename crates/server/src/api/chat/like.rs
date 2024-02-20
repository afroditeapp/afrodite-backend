use axum::{extract::State, Extension, Router};
use model::{AccountId, AccountIdInternal, NotificationEvent, ReceivedLikesPage, SentLikesPage};
use simple_backend::create_counters;

use super::super::{
    db_write,
    utils::{Json, StatusCode},
};
use crate::{api::db_write_multiple, app::{GetAccounts, ReadData, WriteData}};

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
pub async fn post_send_like<S: GetAccounts + WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.post_send_like.incr();

    // TODO: Check is profile public and is age ok.

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    db_write_multiple!(state, move |cmds| {
        let changes = cmds.chat().like_or_match_profile(id, requested_profile).await?;
        cmds.events().handle_chat_state_changes(changes.sender).await?;
        cmds.events().handle_chat_state_changes(changes.receiver).await?;
        Ok(())
    })?;


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
pub async fn delete_like<S: GetAccounts + WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<(), StatusCode> {
    CHAT.delete_like.incr();

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    db_write_multiple!(state, move |cmds| {
        let changes = cmds.chat().delete_like_or_block(id, requested_profile).await?;
        cmds.events().handle_chat_state_changes(changes.sender).await?;
        cmds.events().handle_chat_state_changes(changes.receiver).await?;
        Ok(())
    })?;

    Ok(())
}

pub fn like_router(s: crate::app::S) -> Router {
    use axum::routing::{delete, get, post};

    use crate::app::S;

    Router::new()
        .route(PATH_POST_SEND_LIKE, post(post_send_like::<S>))
        .route(PATH_GET_SENT_LIKES, get(get_sent_likes::<S>))
        .route(PATH_GET_RECEIVED_LIKES, get(get_received_likes::<S>))
        .route(PATH_DELETE_LIKE, delete(delete_like::<S>))
        .with_state(s)
}

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_LIKE_COUNTERS_LIST,
    post_send_like,
    get_sent_likes,
    get_received_likes,
    delete_like,
);
