//! Common routes to all microservices
//!

use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    response::IntoResponse,
    TypedHeader,
};
use error_stack::{ResultExt, Result};
use model::{AccessToken, AccountIdInternal, AuthPair, BackendVersion, RefreshToken, EventToClient, EventToClientInternal};
use simple_backend::web_socket::WebSocketManager;
use tracing::error;
use simple_backend_utils::ContextExt;
pub use utils::api::PATH_CONNECT;

use super::{
    utils::{AccessTokenHeader, Json, StatusCode},
    super::app::{BackendVersionProvider, GetAccessTokens, ReadData, WriteData},
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
pub async fn get_version<S: BackendVersionProvider>(state: S) -> Json<BackendVersion> {
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
    websocket: WebSocketUpgrade,
    TypedHeader(access_token): TypedHeader<AccessTokenHeader>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ws_manager: WebSocketManager,
    state: S,
) -> std::result::Result<impl IntoResponse, StatusCode> {
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
    tokio::select! {
        _ = ws_manager.server_quit_watcher.recv() => (),
        r = handle_socket_result(socket, address, id, &state) => {
            match r {
                Ok(()) => {
                    let result = state.write(move |cmds| async move {
                        cmds.common()
                            .end_connection_session(id, false)
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

    drop(ws_manager.quit_handle);
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
        .ok_or(WebSocketError::DatabaseNoRefreshToken)?
        .bytes()
        .change_context(WebSocketError::InvalidRefreshTokenInDatabase)?;

    // Refresh token check.
    match socket
        .recv()
        .await
        .ok_or(WebSocketError::Receive)?
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
        .write(move |cmds| async move {
            cmds.common()
                .init_connection_session_events(
                    id.uuid,
                )
                .await
        })
        .await
        .change_context(WebSocketError::DatabaseSaveTokens)?;

    send_account_state(&mut socket, id, state).await?;

    loop {
        tokio::select! {
            result = socket.recv() => {
                match result {
                    Some(Err(_)) | None => break,
                    Some(Ok(_)) => continue,
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

    let event: EventToClient = EventToClientInternal::AccountStateChanged { state: current_account.state() }.into();
    let event = serde_json::to_string(
        &event
    )
        .change_context(WebSocketError::Serialize)?;
    socket.send(Message::Text(event))
        .await
        .change_context(WebSocketError::Send)?;

    let event: EventToClient = EventToClientInternal::AccountCapabilitiesChanged { capabilities: current_account.into_capablities() }.into();
    let event = serde_json::to_string(
        &event
    )
        .change_context(WebSocketError::Serialize)?;
    socket.send(Message::Text(event))
        .await
        .change_context(WebSocketError::Send)?;

    Ok(())
}
