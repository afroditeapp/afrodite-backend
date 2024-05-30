//! Common routes to all microservices
//!

use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use model::{
    AccessToken, AccountIdInternal, AuthPair, RefreshToken,
    SyncDataVersionFromClient,
};
use server_data::read::GetReadCommandsCommon;
use server_data_chat::write::GetWriteCommandsChat;
use simple_backend::{create_counters, web_socket::WebSocketManager};
use simple_backend_utils::IntoReportFromString;
use tracing::{error, info};
pub use utils::api::PATH_CONNECT;

use super::utils::{AccessTokenHeader, StatusCode};
use crate::{
    app::{GetAccessTokens, GetConfig, ReadData, WriteData},
    result::{Result, WrappedContextExt, WrappedResultExt},
};

pub mod data_sync;

// TODO(prod): Check access and refresh key lenghts.

// ------------------------- WebSocket -------------------------

/// Connect to server using WebSocket after getting refresh and access tokens.
/// Connection is required as API access is allowed for connected clients.
///
/// Protocol:
/// 1. Client sends version information as Binary message, where
///    - u8: Client WebSocket protocol version (currently 0).
///    - u8: Client type number. (0 = Android, 1 = iOS, 255 = Test mode bot)
///    - u16: Client Major version.
///    - u16: Client Minor version.
///    - u16: Client Patch version.
///
///    The u16 values are in little endian byte order.
/// 2. Client sends current refresh token as Binary message.
/// 3. If server supports the client, the server sends next refresh token
///    as Binary message.
///    If server does not support the client, the server sends Text message
///    and closes the connection.
/// 4. Server sends new access token as Text message.
///    (At this point API can be used.)
/// 5. Client sends list of current data sync versions as Binary message, where
///    items are [u8; 2] and the first u8 of an item is the data type number
///    and the second u8 of an item is the sync version number for that data.
///    If client does not have any version of the data, the client should
///    send 255 as the version number.
///
///    Available data types:
///    - 0: Account
/// 6. Server starts to send JSON events as Text messages.
///
/// The new access token is valid until this WebSocket is closed.
///
#[utoipa::path(
    get,
    path = "/common_api/connect",
    responses(
        (status = 101, description = "Switching protocols."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error. TODO: can be removed?"),
    ),
    security(("access_token" = [])),
)]
pub async fn get_connect_websocket<
    S: WriteData + ReadData + GetAccessTokens + GetConfig + Send + Sync + 'static,
>(
    State(state): State<S>,
    websocket: WebSocketUpgrade,
    TypedHeader(access_token): TypedHeader<AccessTokenHeader>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ws_manager: WebSocketManager,
) -> std::result::Result<impl IntoResponse, StatusCode> {
    CONNECTION.get_connect_websocket.incr();

    // NOTE: This handler does not have authentication layer enabled, so
    // authentication must be done manually.

    let id = state
        .access_token_exists(access_token.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    info!("get_connect_websocket for '{}'", id.id.as_i64());

    Ok(websocket.on_upgrade(move |socket| handle_socket(socket, addr, id, state, ws_manager)))
}

async fn handle_socket<S: WriteData + ReadData + GetConfig>(
    socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: S,
    mut ws_manager: WebSocketManager,
) {
    info!("handle_socket for '{}'", id.id.as_i64());
    let quit_lock = if let Some(quit_lock) = ws_manager.get_ongoing_ws_connection_quit_lock().await
    {
        quit_lock
    } else {
        return;
    };

    tokio::select! {
        _ = ws_manager.server_quit_detected() => {
            info!("Server quit detected, closing WebSocket connection for '{}'", id.id.as_i64());
            // TODO: Probably sessions should be ended when server quits?
            //       Test does this code path work with client.
            let result = state.write(move |cmds| async move {
                cmds.common()
                    .end_connection_session(id)
                    .await
            }).await;

            if let Err(e) = result {
                error!("server quit end_connection_session, {e:?}, for '{id}'");
            }
        },
        r = handle_socket_result(socket, address, id, &state) => {
            match r {
                Ok(()) => {
                    info!("handle_socket_result returned Ok for '{}'", id.id.as_i64());
                    let result = state.write(move |cmds| async move {
                        cmds.common()
                            .end_connection_session(id)
                            .await
                    }).await;

                    if let Err(e) = result {
                        error!("end_connection_session, {e:?}, for '{id}'")
                    }
                },
                Err(e) => {
                    error!("handle_socket_result returned Err {e:?} for id: '{id}'");

                    let result = state.write(move |cmds| async move {
                        cmds.common().logout(id).await
                    }).await;

                    if let Err(e) = result {
                        error!("logout, {e:?}, for '{id}'")
                    }
                }
            }
        }
    }

    drop(quit_lock);
}

#[derive(thiserror::Error, Debug)]
pub enum WebSocketError {
    #[error("Receive error")]
    Receive,
    #[error("Client sent something unsupported")]
    ProtocolError,
    #[error("Client version is unsupported")]
    ClientVersionUnsupported,
    #[error("Received wrong refresh token")]
    ReceiveWrongRefreshToken,
    #[error("Websocket data sending error")]
    Send,
    #[error("Websocket closing failed")]
    Close,
    #[error("Data serialization error")]
    Serialize,

    // Database errors
    #[error("Database: No refresh token")]
    DatabaseNoRefreshToken,
    #[error("Invalid refresh token in database")]
    InvalidRefreshTokenInDatabase,
    #[error("Database: account logout failed")]
    DatabaseLogoutFailed,
    #[error("Database: saving new tokens failed")]
    DatabaseSaveTokens,
    #[error("Database: Account state query failed")]
    DatabaseAccountStateQuery,
    #[error("Database: Chat state query failed")]
    DatabaseChatStateQuery,
    #[error("Database: Profile state query failed")]
    DatabaseProfileStateQuery,
    #[error("Database: Pending messages query failed")]
    DatabasePendingMessagesQuery,
    #[error("Database: Pending notification reset failed")]
    DatabasePendingNotificationReset,

    // Event errors
    #[error("Event channel creation failed")]
    EventChannelCreationFailed,

    // Sync
    #[error("Account data version number reset failed")]
    AccountDataVersionResetFailed,
    #[error("Chat data version number reset failed")]
    ChatDataVersionResetFailed,
    #[error("Profile attributes sync version number reset failed")]
    ProfileAttributesSyncVersionResetFailed,
}

async fn handle_socket_result<S: WriteData + ReadData + GetConfig>(
    mut socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: &S,
) -> Result<(), WebSocketError> {
    info!("handle_socket_result for '{}'", id.id.as_i64());

    // Receive protocol version byte.
    let client_is_supported = match socket
        .recv()
        .await
        .ok_or(WebSocketError::Receive.report())?
        .change_context(WebSocketError::Receive)?
    {
        Message::Binary(version) => {
            match version.as_slice() {
                [0, info_bytes @ ..] => {
                    let info = model::WebSocketClientInfo::parse(info_bytes)
                        .into_error_string(WebSocketError::ProtocolError)?;
                    // TODO: remove after client is tested to work with the
                    // new protocol
                    info!("{:#?}", info);
                    // In the future there is possibility to blacklist some
                    // old client versions.
                    true
                }
                _ => return Err(WebSocketError::ProtocolError.report()),
            }
        }
        _ => return Err(WebSocketError::ProtocolError.report()),
    };

    let current_refresh_token = state
        .read()
        .common()
        .account_refresh_token(id)
        .await
        .change_context(WebSocketError::DatabaseNoRefreshToken)?
        .ok_or(WebSocketError::DatabaseNoRefreshToken.report())?
        .bytes()
        .change_context(WebSocketError::InvalidRefreshTokenInDatabase)?;

    // Refresh token check.
    match socket
        .recv()
        .await
        .ok_or(WebSocketError::Receive.report())?
        .change_context(WebSocketError::Receive)?
    {
        Message::Binary(refresh_token) => {
            if refresh_token != current_refresh_token {
                // Returning error does the logout, so it is not needed here.
                // For this case the logout is needed to prevent refresh
                // token quessing.
                return Err(WebSocketError::ReceiveWrongRefreshToken.report());
            }
        }
        _ => return Err(WebSocketError::ProtocolError.report()),
    };

    if !client_is_supported {
        socket
            .send(Message::Text(String::new()))
            .await
            .change_context(WebSocketError::Send)?;
        socket.close().await.change_context(WebSocketError::Close)?;
        return Err(WebSocketError::ClientVersionUnsupported.report());
    }

    // Refresh check was successful, so the new refresh token can be sent.

    let (new_refresh_token, new_refresh_token_bytes) = RefreshToken::generate_new_with_bytes();
    let new_access_token = AccessToken::generate_new();

    socket
        .send(Message::Binary(new_refresh_token_bytes))
        .await
        .change_context(WebSocketError::Send)?;

    let new_access_token_cloned = new_access_token.clone();
    state
        .write(move |cmds| async move {
            cmds.common()
                .set_new_auth_pair(
                    id,
                    AuthPair {
                        access: new_access_token_cloned,
                        refresh: new_refresh_token,
                    },
                    Some(address),
                )
                .await
        })
        .await
        .change_context(WebSocketError::DatabaseSaveTokens)?;

    socket
        .send(Message::Text(new_access_token.into_string()))
        .await
        .change_context(WebSocketError::Send)?;

    // Receive sync data version list
    let data_sync_versions = match socket
        .recv()
        .await
        .ok_or(WebSocketError::Receive.report())?
        .change_context(WebSocketError::Receive)?
    {
        Message::Binary(sync_data_version_list) => {
            SyncDataVersionFromClient::parse_sync_data_list(&sync_data_version_list)
                .into_error_string(WebSocketError::ProtocolError)?
        }
        _ => return Err(WebSocketError::ProtocolError.report()),
    };

    let mut event_receiver = state
        .write(
            move |cmds| async move { cmds.common().init_connection_session_events(id.uuid).await },
        )
        .await
        .change_context(WebSocketError::DatabaseSaveTokens)?;

    reset_pending_notification(state, id).await?;
    self::data_sync::sync_data_with_client_if_needed(state, &mut socket, id, data_sync_versions)
        .await?;
    self::data_sync::send_new_messages_event_if_needed(state, &mut socket, id).await?;

    // TODO(prod): Remove extra logging from this file.

    loop {
        tokio::select! {
            result = socket.recv() => {
                match result {
                    Some(Err(_)) | None => break,
                    Some(Ok(value)) => {
                        // TODO: Fix possible CPU usage bug here.
                        // Replace continue with break?
                        error!("Unexpected value: {:?}, from: {}", value, address);
                        continue;
                    },
                }
            }
            event = event_receiver.recv() => {
                match event {
                    Some(event) => {
                        let event = serde_json::to_string(&event)
                            .change_context(WebSocketError::Serialize)?;
                        socket.send(Message::Text(event))
                            .await
                            .change_context(WebSocketError::Send)?;
                    },
                    None => {
                        error!("Event receiver channel broken: id: {}, address: {}", id.id.as_i64(), address);
                        // New connection created another event receiver.
                        break;
                    },
                }
            }
        }
    }

    Ok(())
}

pub async fn reset_pending_notification<S: WriteData + GetConfig>(
    state: &S,
    id: AccountIdInternal,
) -> Result<(), WebSocketError> {
    if state.config().components().chat {
        state
            .write(move |cmds| async move {
                cmds.cmds()
                    .chat()
                    .push_notifications()
                    .reset_pending_notification(id)
                    .await
            })
            .await
            .change_context(WebSocketError::DatabasePendingNotificationReset)?;
    }

    Ok(())
}

create_counters!(
    ConnectionCounters,
    CONNECTION,
    CONNECTION_COUNTERS_LIST,
    get_connect_websocket,
);
