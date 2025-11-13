use model::{AccountIdInternal, EventToClientInternal};
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
    let enabled = state
        .config()
        .client_features()
        .map(|v| v.chat.typing_indicator.enabled)
        .unwrap_or_default();

    if !enabled {
        // Ignore event because feature is disabled
        return Ok(());
    }

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
        check_typing_message_rate_limit(state, id, |typing_to_state| match typing_to_state {
            CurrentlyTypingToAccess::Allowed(current) => {
                let current_value = *current;
                *current = Some(typing_to);
                Ok(current_value)
            }
            CurrentlyTypingToAccess::Denied => Err(()),
        })
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
    let enabled = state
        .config()
        .client_features()
        .map(|v| v.chat.typing_indicator.enabled)
        .unwrap_or_default();

    if !enabled {
        // Ignore event because feature is disabled
        return Ok(());
    }

    let value =
        check_typing_message_rate_limit(state, id, |typing_to_state| match typing_to_state {
            CurrentlyTypingToAccess::Allowed(current) => {
                let current_value = *current;
                *current = None;
                Ok(current_value)
            }
            CurrentlyTypingToAccess::Denied => Err(()),
        })
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
    typing_to_state_action: impl FnOnce(CurrentlyTypingToAccess) -> T,
) -> Result<T, WebSocketError> {
    let min_wait_seconds = state
        .config()
        .client_features()
        .map(|v| v.chat.typing_indicator.clone())
        .unwrap_or_default()
        .min_wait_seconds_between_sending_messages;

    let action_return_value = state
        .read()
        .cache_read_write_access()
        .write_cache(id, |cache| {
            let typing_to_state = cache
                .chat
                .currently_typing_to
                .access_typing_to_state(min_wait_seconds);
            Ok(typing_to_state_action(typing_to_state))
        })
        .await
        .change_context(WebSocketError::EventToServerHandlingFailed)?;

    Ok(action_return_value)
}
