use model::{AccountIdInternal, EventToClientInternal, TypingIndicatorConfig};
use server_common::websocket::WebSocketError;
use server_data::{
    app::{GetConfig, ReadData},
    cache::chat::CurrentlyTypingToAccess,
    result::WrappedResultExt,
};
use server_state::S;

use crate::{app::EventManagerProvider, result::Result};

pub async fn handle_typing_start(
    state: &S,
    id: AccountIdInternal,
    typing_to: AccountIdInternal,
) -> Result<(), WebSocketError> {
    let Some(config) = state
        .config()
        .client_features_internal()
        .chat
        .typing_indicator
        .as_ref()
    else {
        // Ignore event because feature is disabled
        return Ok(());
    };

    let is_match = state
        .data_all_access()
        .is_match(id, typing_to)
        .await
        .change_context(WebSocketError::EventToServerHandlingFailed)?;

    if !is_match {
        // Ignore event because chatting is not possible
        return Ok(());
    }

    let value =
        check_typing_message_rate_limit(
            state,
            id,
            config,
            |typing_to_state| match typing_to_state {
                CurrentlyTypingToAccess::Allowed(current) => {
                    let current_value = *current;
                    *current = Some(typing_to);
                    Ok(current_value)
                }
                CurrentlyTypingToAccess::Denied => Err(()),
            },
        )
        .await?;

    match value {
        // Ignore event because wait time is not elapsed
        Err(()) => return Ok(()),
        Ok(None) => (),
        Ok(Some(previous_target)) => {
            if previous_target != typing_to {
                state
                    .event_manager()
                    .send_connected_event(
                        previous_target,
                        EventToClientInternal::TypingStop(id.as_id()),
                    )
                    .await
                    .change_context(WebSocketError::EventToServerHandlingFailed)?;
            }
        }
    }

    state
        .event_manager()
        .send_connected_event(typing_to, EventToClientInternal::TypingStart(id.as_id()))
        .await
        .change_context(WebSocketError::EventToServerHandlingFailed)?;

    Ok(())
}

pub async fn handle_typing_stop(state: &S, id: AccountIdInternal) -> Result<(), WebSocketError> {
    let Some(config) = state
        .config()
        .client_features_internal()
        .chat
        .typing_indicator
        .as_ref()
    else {
        // Ignore event because feature is disabled
        return Ok(());
    };

    let value =
        check_typing_message_rate_limit(
            state,
            id,
            config,
            |typing_to_state| match typing_to_state {
                CurrentlyTypingToAccess::Allowed(current) => {
                    let current_value = *current;
                    *current = None;
                    Ok(current_value)
                }
                CurrentlyTypingToAccess::Denied => Err(()),
            },
        )
        .await?;

    match value {
        // Ignore event because wait time is not elapsed
        Err(()) => return Ok(()),
        Ok(None) => (),
        Ok(Some(previous_target)) => {
            state
                .event_manager()
                .send_connected_event(
                    previous_target,
                    EventToClientInternal::TypingStop(id.as_id()),
                )
                .await
                .change_context(WebSocketError::EventToServerHandlingFailed)?;
        }
    }

    Ok(())
}

async fn check_typing_message_rate_limit<T>(
    state: &S,
    id: AccountIdInternal,
    typing_indicator_config: &TypingIndicatorConfig,
    typing_to_state_action: impl FnOnce(CurrentlyTypingToAccess) -> T,
) -> Result<T, WebSocketError> {
    let action_return_value = state
        .read()
        .cache_read_write_access()
        .write_cache(id, |cache| {
            let typing_to_state = cache.chat.currently_typing_to.access_typing_to_state(
                typing_indicator_config.min_wait_seconds_between_sending_messages_server,
            );
            Ok(typing_to_state_action(typing_to_state))
        })
        .await
        .change_context(WebSocketError::EventToServerHandlingFailed)?;

    Ok(action_return_value)
}

pub async fn handle_check_online_status(
    state: &S,
    id: AccountIdInternal,
    check_account: AccountIdInternal,
    client_is_online: bool,
) -> Result<(), WebSocketError> {
    let Some(config) = state
        .config()
        .client_features_internal()
        .chat
        .check_online_status
        .as_ref()
    else {
        // Ignore event because feature is disabled
        return Ok(());
    };

    let allowed = state
        .read()
        .cache_read_write_access()
        .write_cache(id, |cache| {
            Ok(cache
                .chat
                .check_online_status
                .check_if_allowed(config.min_wait_seconds_between_requests_server))
        })
        .await
        .change_context(WebSocketError::EventToServerHandlingFailed)?;

    if !allowed {
        // Ignore event because min wait time has not elapsed
        return Ok(());
    }

    let is_match = state
        .data_all_access()
        .is_match(id, check_account)
        .await
        .change_context(WebSocketError::EventToServerHandlingFailed)?;

    if !is_match {
        // Ignore event because chatting is not possible
        return Ok(());
    }

    let last_seen_time = state
        .read()
        .cache_read_write_access()
        .read_cache(check_account.as_id(), |cache| {
            Ok(cache.profile.last_seen_time().last_seen_time())
        })
        .await
        .change_context(WebSocketError::EventToServerHandlingFailed)?;

    let actual_is_online = last_seen_time == model::LastSeenTime::ONLINE;

    if client_is_online != actual_is_online {
        state
            .event_manager()
            .send_connected_event(
                id,
                EventToClientInternal::CheckOnlineStatusResponse(last_seen_time),
            )
            .await
            .change_context(WebSocketError::EventToServerHandlingFailed)?;
    }

    Ok(())
}
