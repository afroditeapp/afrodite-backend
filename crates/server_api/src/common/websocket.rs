use axum::extract::ws::{Message, WebSocket};
use model::{
    AccountId, AccountIdInternal, ClientMessageForDataAllCrate, ClientMessageType,
    EventToClientInternal, ScheduledMaintenanceStatus, create_server_binary_message,
};
use model_server_data::ProfileIteratorSessionId;
use server_common::websocket::WebSocketError;
use server_data::{app::ReadData, db_manager::InternalReading, result::WrappedResultExt};
use server_state::S;
use simple_backend::app::GetManagerApi;
use simple_backend_utils::UuidBase64Url;
use utils::minimal_i64;

use super::COMMON;
use crate::result::WrappedContextExt;

pub mod chat;
pub mod profile;
pub mod tracker;

#[derive(Debug, Clone)]
pub enum ClientMessageForServerApiCrate {
    ClearMaintenanceStatusIfPossible,
    RequestResetProfilePaging,
    RequestGetNextProfilePage {
        iterator_session_id: ProfileIteratorSessionId,
    },
    RequestAutomaticProfileSearchResetProfilePaging,
    TypingStart {
        typing_to: AccountId,
    },
    TypingStop,
    RequestCheckOnlineStatus {
        check_account: AccountId,
        is_online: bool,
    },
}

#[derive(Debug, Clone)]
pub enum ClientMessageParsed<'a> {
    ForDataAll(ClientMessageForDataAllCrate<'a>),
    ForServerApi(ClientMessageForServerApiCrate),
}

pub fn parse_client_binary_message(
    binary_message: &[u8],
) -> crate::result::Result<ClientMessageParsed<'_>, WebSocketError> {
    let (message_type_u8, payload) = binary_message
        .split_first()
        .ok_or(WebSocketError::ProtocolError.report())?;

    let message_type = ClientMessageType::try_from(*message_type_u8)
        .map_err(|_| WebSocketError::ProtocolError.report())?;

    match message_type {
        ClientMessageType::SyncVersionList => Ok(ClientMessageParsed::ForDataAll(
            ClientMessageForDataAllCrate::SyncVersionList(payload),
        )),
        ClientMessageType::ClearMaintenanceStatusIfPossible => {
            if !payload.is_empty() {
                return Err(WebSocketError::ProtocolError.report());
            }

            Ok(ClientMessageParsed::ForServerApi(
                ClientMessageForServerApiCrate::ClearMaintenanceStatusIfPossible,
            ))
        }
        ClientMessageType::RequestResetProfilePaging => {
            if !payload.is_empty() {
                return Err(WebSocketError::ProtocolError.report());
            }

            Ok(ClientMessageParsed::ForServerApi(
                ClientMessageForServerApiCrate::RequestResetProfilePaging,
            ))
        }
        ClientMessageType::RequestGetNextProfilePage => {
            let iterator_session_id = parse_profile_iterator_session_id(payload)?;
            Ok(ClientMessageParsed::ForServerApi(
                ClientMessageForServerApiCrate::RequestGetNextProfilePage {
                    iterator_session_id,
                },
            ))
        }
        ClientMessageType::RequestAutomaticProfileSearchResetProfilePaging => {
            if !payload.is_empty() {
                return Err(WebSocketError::ProtocolError.report());
            }

            Ok(ClientMessageParsed::ForServerApi(
                ClientMessageForServerApiCrate::RequestAutomaticProfileSearchResetProfilePaging,
            ))
        }
        ClientMessageType::TypingStart => {
            let typing_to = parse_account_id(payload)?;
            Ok(ClientMessageParsed::ForServerApi(
                ClientMessageForServerApiCrate::TypingStart { typing_to },
            ))
        }
        ClientMessageType::TypingStop => {
            if !payload.is_empty() {
                return Err(WebSocketError::ProtocolError.report());
            }

            Ok(ClientMessageParsed::ForServerApi(
                ClientMessageForServerApiCrate::TypingStop,
            ))
        }
        ClientMessageType::RequestCheckOnlineStatus => {
            let (check_account_payload, is_online_payload) = payload
                .split_at_checked(16)
                .ok_or(WebSocketError::ProtocolError.report())?;
            let is_online = is_online_payload.first().copied().unwrap_or_default() != 0;

            let check_account = parse_account_id(check_account_payload)?;

            Ok(ClientMessageParsed::ForServerApi(
                ClientMessageForServerApiCrate::RequestCheckOnlineStatus {
                    check_account,
                    is_online,
                },
            ))
        }
    }
}

fn parse_account_id(payload: &[u8]) -> crate::result::Result<AccountId, WebSocketError> {
    let bytes: [u8; 16] = payload
        .try_into()
        .map_err(|_| WebSocketError::ProtocolError.report())?;

    Ok(AccountId::new_base_64_url(UuidBase64Url::from_bytes(bytes)))
}

fn parse_profile_iterator_session_id(
    payload: &[u8],
) -> crate::result::Result<ProfileIteratorSessionId, WebSocketError> {
    let mut iterator = payload.iter().copied();
    let value = minimal_i64::parse_minimal_i64_from_iter(&mut iterator)
        .ok_or(WebSocketError::ProtocolError.report())?;

    if iterator.next().is_some() {
        return Err(WebSocketError::ProtocolError.report());
    }

    Ok(ProfileIteratorSessionId::from_i64(value))
}

/// Errors which can cause log spam are ignored so
/// logging the returned error is safe.
pub async fn handle_message_from_client(
    state: &S,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    msg: ClientMessageForServerApiCrate,
) -> crate::result::Result<(), WebSocketError> {
    match msg {
        ClientMessageForServerApiCrate::ClearMaintenanceStatusIfPossible => {
            if state
                .manager_api_client()
                .maintenance_status()
                .await
                .is_empty()
            {
                send_event(
                    socket,
                    EventToClientInternal::ScheduledMaintenanceStatus(
                        ScheduledMaintenanceStatus::default(),
                    ),
                )
                .await?;
            }
            Ok(())
        }
        ClientMessageForServerApiCrate::RequestResetProfilePaging => {
            profile::handle_reset_profile_paging(state, socket, id).await
        }
        ClientMessageForServerApiCrate::RequestGetNextProfilePage {
            iterator_session_id,
        } => profile::handle_get_next_profile_page(state, socket, id, iterator_session_id).await,
        ClientMessageForServerApiCrate::RequestAutomaticProfileSearchResetProfilePaging => {
            profile::handle_automatic_profile_search_reset_profile_paging(state, socket, id).await
        }
        ClientMessageForServerApiCrate::TypingStart { typing_to } => {
            COMMON.event_to_server_typing_start.incr();
            let Some(typing_to) = state
                .read()
                .cache()
                .to_account_id_internal_optional(typing_to)
                .await
            else {
                // Ignore invalid account ID
                return Ok(());
            };
            chat::handle_typing_start(state, id, typing_to).await
        }
        ClientMessageForServerApiCrate::TypingStop => {
            COMMON.event_to_server_typing_stop.incr();
            chat::handle_typing_stop(state, id).await
        }
        ClientMessageForServerApiCrate::RequestCheckOnlineStatus {
            check_account,
            is_online,
        } => {
            COMMON.event_to_server_check_online_status.incr();
            let Some(check_account) = state
                .read()
                .cache()
                .to_account_id_internal_optional(check_account)
                .await
            else {
                // Ignore invalid account ID
                return Ok(());
            };
            chat::handle_check_online_status(state, id, check_account, is_online).await
        }
    }
}

pub async fn send_event(
    socket: &mut WebSocket,
    event: EventToClientInternal,
) -> crate::result::Result<(), WebSocketError> {
    socket
        .send(Message::Binary(create_server_binary_message(&event).into()))
        .await
        .change_context(WebSocketError::Send)?;

    Ok(())
}
