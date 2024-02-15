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
    AccessToken, AccountIdInternal, AccountSyncVersion, AccountSyncVersionFromClient, AuthPair, BackendVersion, EventToClient, EventToClientInternal, RefreshToken, SpecialEventToClient
};
use simple_backend::{create_counters, web_socket::WebSocketManager};
use tracing::error;
pub use utils::api::PATH_CONNECT;

use super::{
    super::app::{BackendVersionProvider, GetAccessTokens, ReadData, WriteData},
    utils::{AccessTokenHeader, Json, StatusCode},
};
use crate::result::{Result, WrappedContextExt, WrappedResultExt};

pub const PATH_GET_VERSION: &str = "/common_api/version";

/// Get backend version.
#[utoipa::path(
    get,
    path = "/common_api/version",
    security(),
    responses(
        (status = 200, description = "Version information.", body = BackendVersion),
    )
)]
pub async fn get_version<S: BackendVersionProvider>(
    State(state): State<S>,
) -> Json<BackendVersion> {
    COMMON.get_version.incr();
    state.backend_version().into()
}

// TODO(prod): Check access and refresh key lenghts.

// ------------------------- WebSocket -------------------------

/// Connect to server using WebSocket after getting refresh and access tokens.
/// Connection is required as API access is allowed for connected clients.
///
/// Protocol:
/// 1. Client sends protocol version byte as Binary message.
/// 2. Client sends current refresh token as Binary message.
/// 3. Server sends next refresh token as Binary message.
/// 4. Server sends new access token as Text message.
///    (At this point API can be used.)
/// 5. Client sends account state sync version as Binary message.
///    The version is i64 value with little endian byte order.
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
    S: WriteData + ReadData + GetAccessTokens + Send + Sync + 'static,
>(
    State(state): State<S>,
    websocket: WebSocketUpgrade,
    TypedHeader(access_token): TypedHeader<AccessTokenHeader>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ws_manager: WebSocketManager,
) -> std::result::Result<impl IntoResponse, StatusCode> {
    COMMON.get_connect_websocket.incr();

    // NOTE: This handler does not have authentication layer enabled, so
    // authentication must be done manually.

    let id = state
        .access_tokens()
        .access_token_exists(access_token.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(websocket.on_upgrade(move |socket| handle_socket(socket, addr, id, state, ws_manager)))
}

async fn handle_socket<S: WriteData + ReadData>(
    socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: S,
    mut ws_manager: WebSocketManager,
) {
    let quit_lock = if let Some(quit_lock) = ws_manager.get_ongoing_ws_connection_quit_lock().await
    {
        quit_lock
    } else {
        return;
    };

    tokio::select! {
        _ = ws_manager.server_quit_detected() => {
            // TODO: Probably sessions should be ended when server quits?
            //       Test does this code path work with client.
            let result = state.write(move |cmds| async move {
                cmds.common()
                    .end_connection_session(id)
                    .await
            }).await;

            if let Err(e) = result {
                error!("server quit end_connection_session, {e:?}")
            }
        },
        r = handle_socket_result(socket, address, id, &state) => {
            match r {
                Ok(()) => {
                    let result = state.write(move |cmds| async move {
                        cmds.common()
                            .end_connection_session(id)
                            .await
                    }).await;

                    if let Err(e) = result {
                        error!("end_connection_session, {e:?}")
                    }
                },
                Err(e) => {
                    error!("handle_socket_result: {e:?}");

                    let result = state.write(move |cmds| async move {
                        cmds.common().logout(id).await
                    }).await;

                    if let Err(e) = result {
                        error!("logout, {e:?}")
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
    #[error("Received something else than refresh token")]
    ReceiveMissingRefreshToken,
    #[error("Received something else than supported protocol version")]
    ReceiveUnsupportedProtocolVersion,
    #[error("Received unsupported account state sync version")]
    ReceiveUnsupportedSyncVersion,
    #[error("Received wrong refresh token")]
    ReceiveWrongRefreshToken,
    #[error("Websocket data sending error")]
    Send,
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

    // Event errors
    #[error("Event channel creation failed")]
    EventChannelCreationFailed,
}

async fn handle_socket_result<S: WriteData + ReadData>(
    mut socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: &S,
) -> Result<(), WebSocketError> {

    // Receive protocol version byte.
    match socket
        .recv()
        .await
        .ok_or(WebSocketError::Receive.report())?
        .change_context(WebSocketError::Receive)?
        {
            Message::Binary(version) => {
                if let [1] = version.as_slice() {
                    // Supported version
                } else {
                    return Err(WebSocketError::ReceiveUnsupportedProtocolVersion.report());
                }
            }
            _ => return Err(WebSocketError::ReceiveUnsupportedProtocolVersion.report()),
        };

    let current_refresh_token = state
        .read()
        .account()
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
        _ => return Err(WebSocketError::ReceiveMissingRefreshToken.report()),
    };

    // Refresh token matched

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

    // Receive account state sync version
    let sync_version = match socket
        .recv()
        .await
        .ok_or(WebSocketError::Receive.report())?
        .change_context(WebSocketError::Receive)?
        {
            Message::Binary(version) => {
                let array: [u8; 8] = TryInto::<[u8; 8]>::try_into(version)
                    .map_err(|_| WebSocketError::ReceiveUnsupportedSyncVersion.report())?;
                AccountSyncVersionFromClient::new(i64::from_le_bytes(array))
            }
            _ => return Err(WebSocketError::ReceiveUnsupportedSyncVersion.report()),
        };

    let mut event_receiver = state
        .write(
            move |cmds| async move { cmds.common().init_connection_session_events(id.uuid).await },
        )
        .await
        .change_context(WebSocketError::DatabaseSaveTokens)?;

    send_account_state(state, &mut socket, id, sync_version).await?;

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
                    None => (),
                }
            }
        }
    }

    Ok(())
}

async fn send_account_state<S: WriteData + ReadData>(
    state: &S,
    socket: &mut WebSocket,
    id: AccountIdInternal,
    sync_version: AccountSyncVersionFromClient,
) -> Result<(), WebSocketError> {
    let current_account = state
        .read()
        .common()
        .account(id)
        .await
        .change_context(WebSocketError::DatabaseAccountStateQuery)?;

    if !current_account.sync_version().sync_required(sync_version) {
        return Ok(());
    }

    send_event(
        socket,
        EventToClientInternal::AccountStateChanged(
            current_account.state()
        )
    ).await?;

    send_event(
        socket,
        EventToClientInternal::AccountCapabilitiesChanged(
            current_account.capablities().clone()
        )
    ).await?;

    send_event(
        socket,
        EventToClientInternal::ProfileVisibilityChanged(
            current_account.profile_visibility()
        )
    ).await?;

    // AccountSyncNumber
    // This must be the last to make sure that client has
    // reveived all sync data.
    send_event(
        socket,
        SpecialEventToClient::AccountSyncVersionChanged(
            current_account.sync_version()
        )
    ).await?;

    Ok(())
}

async fn send_event(
    socket: &mut WebSocket,
    event: impl Into<EventToClient>,
) -> Result<(), WebSocketError> {
    let event: EventToClient = event.into();
    let event = serde_json::to_string(&event).change_context(WebSocketError::Serialize)?;
    socket
        .send(Message::Text(event))
        .await
        .change_context(WebSocketError::Send)?;

    Ok(())
}

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_COUNTERS_LIST,
    get_version,
    get_connect_websocket,
);
