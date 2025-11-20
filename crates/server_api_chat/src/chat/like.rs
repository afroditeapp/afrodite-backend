use axum::{Extension, extract::State};
use model::AccountState;
use model_chat::{
    AccountId, AccountIdInternal, AccountInteractionState, CurrentAccountInteractionState,
    DailyLikesLeft, LimitedActionStatus, MarkReceivedLikesViewed, NewReceivedLikesCountResult,
    PushNotificationFlags, ReceivedLikesIteratorState, ReceivedLikesPage,
    ResetReceivedLikesIteratorResult, SendLikeResult,
};
use server_api::{
    S,
    app::{ApiUsageTrackerProvider, EventManagerProvider, GetConfig},
    create_open_api_router,
};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::{
    app::{GetAccounts, ReadData, WriteData},
    db_write,
};

const PATH_POST_SEND_LIKE: &str = "/chat_api/send_like";

/// Send a like to some account. If both will like each other, then
/// the accounts will be a match.
///
/// This route might update [model_chat::DailyLikesLeft] and WebSocket event
/// about the update is not sent because this route returns the new value.
///
/// The like sending is allowed even if accounts aren't a match when
/// considering age and gender preferences. This is because changing
/// the preferences isn't limited.
///
/// # Access
/// * [AccountState::Normal]
#[utoipa::path(
    post,
    path = PATH_POST_SEND_LIKE,
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success.", body = SendLikeResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_send_like(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Json(requested_account): Json<AccountId>,
) -> Result<Json<SendLikeResult>, StatusCode> {
    CHAT.post_send_like.incr();
    state
        .api_usage_tracker()
        .incr(id, |u| &u.post_send_like)
        .await;

    if account_state != AccountState::Normal {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let requested_account = state.get_internal_id(requested_account).await?;

    let r = db_write!(state, move |cmds| {
        let current_interaction = cmds
            .read()
            .chat()
            .account_interaction(id, requested_account)
            .await?;
        if let Some(current_interaction) = current_interaction {
            match current_interaction.state_number {
                AccountInteractionState::Empty => (),
                AccountInteractionState::Match => {
                    return Ok(SendLikeResult::error_account_interaction_state_mismatch(
                        CurrentAccountInteractionState::Match,
                    ));
                }
                AccountInteractionState::Like => {
                    if current_interaction.account_id_sender == Some(id.into_db_id()) {
                        return Ok(SendLikeResult::error_account_interaction_state_mismatch(
                            CurrentAccountInteractionState::LikeSent,
                        ));
                    }
                }
            }
        }

        let unlimited_likes = cmds
            .read()
            .chat()
            .is_unlimited_likes_enabled(requested_account)
            .await?;

        let like_sending_limit_enabled = cmds
            .config()
            .client_features_internal()
            .likes
            .daily
            .is_some();

        let no_like_limit = !like_sending_limit_enabled || unlimited_likes;

        let allow_action = if no_like_limit {
            true
        } else {
            cmds.read()
                .chat()
                .limits()
                .daily_likes_left_internal(id)
                .await?
                .likes_left
                > 0
        };

        if allow_action {
            let changes = cmds
                .chat()
                .like_or_match_profile(id, requested_account)
                .await?;
            cmds.events()
                .handle_chat_state_changes(&changes.sender)
                .await?;
            cmds.events()
                .handle_chat_state_changes(&changes.receiver)
                .await?;
        }

        if !no_like_limit && allow_action {
            cmds.chat().limits().decrement_daily_likes_left(id).await?;
        }

        let daily_likes = cmds
            .read()
            .chat()
            .limits()
            .daily_likes_left_internal(id)
            .await?;

        let status = if no_like_limit {
            LimitedActionStatus::Success
        } else if !allow_action {
            LimitedActionStatus::FailureLimitAlreadyReached
        } else if daily_likes.likes_left == 0 {
            LimitedActionStatus::SuccessAndLimitReached
        } else {
            LimitedActionStatus::Success
        };

        Ok(SendLikeResult::successful(status, daily_likes.into()))
    })?;

    Ok(r.into())
}

const PATH_POST_GET_NEW_RECEIVED_LIKES_COUNT: &str = "/chat_api/new_received_likes_count";

#[utoipa::path(
    post,
    path = PATH_POST_GET_NEW_RECEIVED_LIKES_COUNT,
    responses(
        (status = 200, description = "Success.", body = NewReceivedLikesCountResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_new_received_likes_count(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<NewReceivedLikesCountResult>, StatusCode> {
    CHAT.post_get_new_received_likes_count.incr();

    let mut info = state
        .read()
        .chat()
        .chat_state(id)
        .await?
        .new_received_likes_info();

    let visibility = state
        .event_manager()
        .remove_pending_push_notification_flags_from_cache(
            id,
            PushNotificationFlags::RECEIVED_LIKES_CHANGED,
        )
        .await;
    info.h = visibility.hidden;

    Ok(info.into())
}

const PATH_POST_RESET_NEW_RECEIVED_LIKES_COUNT: &str = "/chat_api/reset_new_received_likes_count";

#[utoipa::path(
    post,
    path = PATH_POST_RESET_NEW_RECEIVED_LIKES_COUNT,
    responses(
        (status = 200, description = "Successfull.", body = NewReceivedLikesCountResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_reset_new_received_likes_count(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<NewReceivedLikesCountResult>, StatusCode> {
    CHAT.post_reset_new_received_likes_count.incr();
    let r = db_write!(state, move |cmds| {
        cmds.chat()
            .handle_reset_new_received_likes_count(account_id)
            .await
    })?;
    Ok(r.into())
}

const PATH_POST_RESET_RECEIVED_LIKES_PAGING: &str = "/chat_api/received_likes/reset";

#[utoipa::path(
    post,
    path = PATH_POST_RESET_RECEIVED_LIKES_PAGING,
    responses(
        (status = 200, description = "Successfull.", body = ResetReceivedLikesIteratorResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_reset_received_likes_paging(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ResetReceivedLikesIteratorResult>, StatusCode> {
    CHAT.post_reset_received_likes_paging.incr();
    let r = db_write!(state, move |cmds| {
        cmds.chat()
            .handle_reset_received_likes_iterator(account_id)
            .await
    })?;
    Ok(r.into())
}

const PATH_POST_GET_RECEIVED_LIKES_PAGE: &str = "/chat_api/received_likes";

/// Get next page of received likes. If the page is empty there is no more
/// received likes available.
///
/// Profile will not be returned if:
/// - Profile is blocked
/// - Profile is a match
#[utoipa::path(
    post,
    path = PATH_POST_GET_RECEIVED_LIKES_PAGE,
    request_body(content = ReceivedLikesIteratorState),
    responses(
        (status = 200, description = "Success.", body = ReceivedLikesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_received_likes_page(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(iterator_state): Json<ReceivedLikesIteratorState>,
) -> Result<Json<ReceivedLikesPage>, StatusCode> {
    CHAT.post_get_received_likes_page.incr();
    let v = state
        .read()
        .chat()
        .received_likes_page(account_id, iterator_state)
        .await?;
    Ok(v.into())
}

const PATH_POST_MARK_RECEIVED_LIKES_VIEWED: &str = "/chat_api/mark_received_likes_viewed";

#[utoipa::path(
    post,
    path = PATH_POST_MARK_RECEIVED_LIKES_VIEWED,
    request_body(content = MarkReceivedLikesViewed),
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_mark_received_likes_viewed(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(likes): Json<MarkReceivedLikesViewed>,
) -> Result<(), StatusCode> {
    CHAT.post_mark_received_likes_viewed.incr();
    db_write!(state, move |cmds| {
        cmds.chat()
            .mark_received_likes_viewed(account_id, likes.v)
            .await
    })?;
    Ok(())
}

const PATH_GET_DAILY_LIKES_LEFT: &str = "/chat_api/daily_likes_left";

/// Get daily likes left value.
#[utoipa::path(
    get,
    path = PATH_GET_DAILY_LIKES_LEFT,
    responses(
        (status = 200, description = "Success.", body = DailyLikesLeft),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_daily_likes_left(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<DailyLikesLeft>, StatusCode> {
    CHAT.get_daily_likes_left.incr();

    let likes = state
        .read()
        .chat()
        .limits()
        .daily_likes_left_internal(id)
        .await?;
    let likes: DailyLikesLeft = likes.into();
    Ok(likes.into())
}

create_open_api_router!(
        fn router_like,
        post_send_like,
        post_get_new_received_likes_count,
        post_reset_new_received_likes_count,
        post_reset_received_likes_paging,
        post_get_received_likes_page,
        post_mark_received_likes_viewed,
        get_daily_likes_left,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_LIKE_COUNTERS_LIST,
    post_send_like,
    post_get_new_received_likes_count,
    post_reset_new_received_likes_count,
    post_reset_received_likes_paging,
    post_get_received_likes_page,
    post_mark_received_likes_viewed,
    get_daily_likes_left,
);
