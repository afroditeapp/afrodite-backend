use axum::{extract::State, Extension};
use model_chat::{
    AccountId, AccountIdInternal, AccountInteractionState, CurrentAccountInteractionState,
    DeleteLikeResult, LimitedActionStatus, NewReceivedLikesCount, NewReceivedLikesCountResult,
    PageItemCountForNewLikes, PendingNotificationFlags, ReceivedLikesIteratorSessionId,
    ReceivedLikesPage, ResetReceivedLikesIteratorResult, SendLikeResult, SentLikesPage,
};
use server_api::{app::{ApiUsageTrackerProvider, EventManagerProvider, GetConfig}, create_open_api_router, S};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::{
    app::{GetAccounts, ReadData, WriteData},
    db_write_multiple,
};

const PATH_POST_SEND_LIKE: &str = "/chat_api/send_like";

/// Send a like to some account. If both will like each other, then
/// the accounts will be a match.
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
    Json(requested_profile): Json<AccountId>,
) -> Result<Json<SendLikeResult>, StatusCode> {
    CHAT.post_send_like.incr();
    state.api_usage_tracker().incr(id, |u| &u.post_send_like).await;

    // TODO(prod): Check is profile public and is age ok.

    let requested_profile = state.get_internal_id(requested_profile).await?;

    let r = db_write_multiple!(state, move |cmds| {
        let current_interaction = cmds
            .read()
            .chat()
            .account_interaction(id, requested_profile)
            .await?;
        if let Some(current_interaction) = current_interaction {
            match current_interaction.state_number {
                AccountInteractionState::Empty => (),
                AccountInteractionState::Match => {
                    return Ok(SendLikeResult::error_account_interaction_state_mismatch(
                        CurrentAccountInteractionState::Match,
                    ))
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

        let unlimited_likes_enabled_for_both = cmds
            .read()
            .chat()
            .unlimited_likes_are_enabled_for_both(id, requested_profile)
            .await?;

        let allow_action = if unlimited_likes_enabled_for_both {
            true
        } else {
            cmds.chat()
                .modify_chat_limits(id, |limits| {
                    limits.like_limit.is_limit_not_reached(cmds.config())
                })
                .await??
        };

        if allow_action {
            let changes = cmds
                .chat()
                .like_or_match_profile(id, requested_profile)
                .await?;
            cmds.events()
                .handle_chat_state_changes(
                    &changes.sender,
                    cmds.read().chat().notification().likes(changes.sender.id).await?,
                )
                .await?;
            cmds.events()
                .handle_chat_state_changes(
                    &changes.receiver,
                    cmds.read().chat().notification().likes(changes.receiver.id).await?,
                )
                .await?;
        }

        let status = if unlimited_likes_enabled_for_both {
            LimitedActionStatus::Success
        } else {
            cmds.chat()
                .modify_chat_limits(id, |limits| {
                    limits.like_limit.increment_if_possible(cmds.config())
                })
                .await??
                .to_action_status()
        };
        Ok(SendLikeResult::successful(status))
    })?;

    Ok(r.into())
}

const PATH_GET_SENT_LIKES: &str = "/chat_api/sent_likes";

/// Get sent likes.
///
/// Profile will not be returned if:
///
/// - Profile is hidden (not public)
/// - Profile is a match
#[utoipa::path(
    get,
    path = PATH_GET_SENT_LIKES,
    responses(
        (status = 200, description = "Success.", body = SentLikesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_sent_likes(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<SentLikesPage>, StatusCode> {
    CHAT.get_sent_likes.incr();

    let page = state.read().chat().all_sent_likes(id).await?;
    Ok(page.into())
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

    let chat_state = state.read().chat().chat_state(id).await?;
    let r = NewReceivedLikesCountResult {
        v: chat_state.received_likes_sync_version,
        c: chat_state.new_received_likes_count,
    };

    state
        .event_manager()
        .remove_specific_pending_notification_flags_from_cache(
            id,
            PendingNotificationFlags::RECEIVED_LIKES_CHANGED,
        )
        .await;

    Ok(r.into())
}

// TODO(prod): Add pagination to chat related lists. Pagination
// iterator shows latest items first. Some ID key will be used as a
// iterator starting point so that newer items do not make older items
// to appear again in next pages. The sync version will be changed to
// invalidate the pagination iterator.

// TODO(prod): Remove received blocks from API and make liking and message
// sending to seem like it succeeded even if the profile owner has blocked
// you.

// TODO(prod): Encryption public key management for chats.

// TODO(prod): Add endless likes support. If user has enabled endless likes, it
// will be possible to send likes without any limits to those users who also
// have the same setting enabled. Profile needs info are endless likes enabled.

// TODO(prod): Limit likes so that only one normal like can be sent per day.

// TODO(prod): Add profile last seen time to profiles.

// TODO(prod): Store date and time when account was created.

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
    let (iterator_session_id, new_version) = db_write_multiple!(state, move |cmds| {
        cmds.chat().handle_reset_received_likes_iterator(account_id).await
    })?;
    let r = ResetReceivedLikesIteratorResult {
        v: new_version,
        c: NewReceivedLikesCount::default(),
        s: iterator_session_id.into(),
    };

    Ok(r.into())
}

const PATH_POST_GET_NEXT_RECEIVED_LIKES_PAGE: &str = "/chat_api/received_likes";

/// Update received likes iterator and get next page
/// of received likes. If the page is empty there is no more
/// received likes available.
///
/// Profile will not be returned if:
/// - Profile is blocked
/// - Profile is a match
#[utoipa::path(
    post,
    path = PATH_POST_GET_NEXT_RECEIVED_LIKES_PAGE,
    request_body(content = ReceivedLikesIteratorSessionId),
    responses(
        (status = 200, description = "Success.", body = ReceivedLikesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_next_received_likes_page(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(iterator_session_id): Json<ReceivedLikesIteratorSessionId>,
) -> Result<Json<ReceivedLikesPage>, StatusCode> {
    CHAT.post_get_next_received_likes_page.incr();

    let data = state
        .concurrent_write_profile_blocking(account_id.as_id(), move |cmds| {
            cmds.next_received_likes_iterator_state(account_id, iterator_session_id)
        })
        .await??;

    if let Some(data) = data {
        // Received likes iterator session ID was valid
        let (profiles, new_likes_count) = state
            .read()
            .chat()
            .received_likes_page(account_id, data)
            .await?;
        Ok(ReceivedLikesPage {
            n: new_likes_count,
            p: profiles,
            error_invalid_iterator_session_id: false,
        }
        .into())
    } else {
        Ok(ReceivedLikesPage {
            n: PageItemCountForNewLikes::default(),
            p: vec![],
            error_invalid_iterator_session_id: true,
        }
        .into())
    }
}

const PATH_DELETE_LIKE: &str = "/chat_api/delete_like";

/// Delete sent like.
///
/// Delete will not work if profile is a match.
#[utoipa::path(
    delete,
    path = PATH_DELETE_LIKE,
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Success.", body = DeleteLikeResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_like(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(requested_profile): Json<AccountId>,
) -> Result<Json<DeleteLikeResult>, StatusCode> {
    CHAT.delete_like.incr();

    let requested_profile = state.get_internal_id(requested_profile).await?;

    let r = db_write_multiple!(state, move |cmds| {
        let current_interaction = cmds
            .read()
            .chat()
            .account_interaction(id, requested_profile)
            .await?;
        if let Some(current_interaction) = current_interaction {
            match current_interaction.state_number {
                AccountInteractionState::Empty => {
                    return Ok(DeleteLikeResult::error_account_interaction_state_mismatch(
                        CurrentAccountInteractionState::Empty,
                    ))
                }
                AccountInteractionState::Match => {
                    return Ok(DeleteLikeResult::error_account_interaction_state_mismatch(
                        CurrentAccountInteractionState::Match,
                    ))
                }
                AccountInteractionState::Like => (),
            }
        }

        let like_deleted_once_before = cmds
            .read()
            .chat()
            .account_interaction(id, requested_profile)
            .await?
            .map(|v| v.account_already_deleted_like(id))
            .unwrap_or_default();

        if like_deleted_once_before {
            return Ok(DeleteLikeResult::error_delete_already_done_once_before());
        }

        let changes = cmds.chat().delete_like(id, requested_profile).await?;
        cmds.events()
            .handle_chat_state_changes(
                &changes.sender,
                cmds.read().chat().notification().likes(changes.sender.id).await?,
            )
            .await?;
        cmds.events()
            .handle_chat_state_changes(
                &changes.receiver,
                cmds.read().chat().notification().likes(changes.receiver.id).await?,
            )
            .await?;

        Ok(DeleteLikeResult::success())
    })?;

    Ok(r.into())
}

create_open_api_router!(
        fn router_like,
        post_send_like,
        get_sent_likes,
        post_get_new_received_likes_count,
        post_reset_received_likes_paging,
        post_get_next_received_likes_page,
        delete_like,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_LIKE_COUNTERS_LIST,
    post_send_like,
    get_sent_likes,
    post_get_new_received_likes_count,
    post_reset_received_likes_paging,
    post_get_next_received_likes_page,
    delete_like,
);
