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
use crate::result::{Result, WrappedContextExt, WrappedResultExt};
use model::{
    AccessToken, AccountIdInternal, AuthPair, BackendVersion, EventToClient, EventToClientInternal,
    RefreshToken,
};
use simple_backend::{create_counters, web_socket::WebSocketManager};
use tracing::error;
pub use utils::api::PATH_CONNECT;

use super::{
    super::app::{BackendVersionProvider, GetAccessTokens, ReadData, WriteData},
    utils::{AccessTokenHeader, Json, StatusCode},
};

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

// ------------------------- WebSocket -------------------------

/// Connect to server using WebSocket after getting refresh and access tokens.
/// Connection is required as API access is allowed for connected clients.
///
/// Send the current refersh token as Binary. The server will send the next
/// refresh token (Binary) and after that the new access token (Text). After
/// that API can be used.
///
/// The access token is valid until this WebSocket is closed. Server might send
/// events as Text which is JSON.
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
    let quit_lock = if let Some(quit_lock) = ws_manager.get_ongoing_ws_connection_quit_lock().await {
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
    // TODO: add close server notification select? Or probably not needed as
    // server should shutdown after main future?

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
                state
                    .write(move |cmds| async move { cmds.common().logout(id).await })
                    .await
                    .change_context(WebSocketError::DatabaseLogoutFailed)?;
                return Ok(());
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

    let mut event_receiver = state
        .write(
            move |cmds| async move { cmds.common().init_connection_session_events(id.uuid).await },
        )
        .await
        .change_context(WebSocketError::DatabaseSaveTokens)?;

    send_account_state(&mut socket, id, state).await?;

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
    socket: &mut WebSocket,
    id: AccountIdInternal,
    state: &S,
) -> Result<(), WebSocketError> {
    let current_account = state
        .read()
        .account()
        .account(id)
        .await
        .change_context(WebSocketError::DatabaseAccountStateQuery)?;

    let event: EventToClient = EventToClientInternal::AccountStateChanged {
        state: current_account.state(),
    }
    .into();
    let event = serde_json::to_string(&event).change_context(WebSocketError::Serialize)?;
    socket
        .send(Message::Text(event))
        .await
        .change_context(WebSocketError::Send)?;

    let event: EventToClient = EventToClientInternal::AccountCapabilitiesChanged {
        capabilities: current_account.into_capablities(),
    }
    .into();
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
