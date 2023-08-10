//! Common routes to all microservices
//!

// TODO: add app version route

use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    response::IntoResponse,
    Json, TypedHeader,
};

use futures::StreamExt;
use hyper::StatusCode;

use utoipa::ToSchema;

use crate::app::connection::WebSocketManager;
use utils::IntoReportExt;

use super::{BackendVersionProvider, WriteData};

use model::BackendVersion;

use model::{AccountIdInternal, ApiKey, AuthPair, RefreshToken};

use tracing::error;

use super::{utils::ApiKeyHeader, GetApiKeys, ReadDatabase};

use error_stack::{IntoReport, Result, ResultExt};

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

pub const PATH_CONNECT: &str = "/common_api/connect";

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
    security(("api_key" = [])),
)]
pub async fn get_connect_websocket<
    S: WriteData + ReadDatabase + GetApiKeys + Send + Sync + 'static,
>(
    websocket: WebSocketUpgrade,
    TypedHeader(access_token): TypedHeader<ApiKeyHeader>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ws_manager: WebSocketManager,
    state: S,
) -> std::result::Result<impl IntoResponse, StatusCode> {
    // NOTE: This handler does not have authentication layer enabled, so
    // authentication must be done manually.

    let id = state
        .api_keys()
        .api_key_exists(access_token.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(websocket.on_upgrade(move |socket| handle_socket(socket, addr, id, state, ws_manager)))
}

async fn handle_socket<S: WriteData + ReadDatabase>(
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
                    let write = state.write(move |cmds| async move {
                        cmds.common()
                            .end_connection_session(id, false)
                            .await
                    }).await;

                    match write {
                        Ok(()) => (),
                        Err(e) => {
                            error!("WebSocket: {e:?}");
                        }
                    }
                },
                Err(e) => {
                    error!("WebSocket: {e:?}");
                    let result = state.write(move |cmds| async move { cmds.common().logout(id).await }).await;
                    match result {
                        Ok(()) => (),
                        Err(e) => {
                            error!("WebSocket: {e:?}");
                        }
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
    #[error("Send error")]
    Send,

    // Database errors
    #[error("Database: No refresh token")]
    DatabaseNoRefreshToken,
    #[error("Invalid refresh token in database")]
    InvalidRefreshTokenInDatabase,
    #[error("Database: account logout failed")]
    DatabaseLogoutFailed,
    #[error("Database: saving new tokens failed")]
    DatabaseSaveTokens,
}

async fn handle_socket_result<S: WriteData + ReadDatabase>(
    mut socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: &S,
) -> Result<(), WebSocketError> {
    // TODO: add close server notification select? Or probably not needed as
    // server should shutdown after main future?

    let current_refresh_token = state
        .read_database()
        .account()
        .account_refresh_token(id)
        .await
        .change_context(WebSocketError::DatabaseNoRefreshToken)?
        .ok_or(WebSocketError::DatabaseNoRefreshToken)?
        .bytes()
        .into_error(WebSocketError::InvalidRefreshTokenInDatabase)?;

    // Refresh token check.
    match socket
        .recv()
        .await
        .ok_or(WebSocketError::Receive)?
        .into_error(WebSocketError::Receive)?
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
        _ => return Err(WebSocketError::ReceiveMissingRefreshToken).into_report(),
    };

    // Refresh token matched

    let (new_refresh_token, new_refresh_token_bytes) = RefreshToken::generate_new_with_bytes();
    let new_access_token = ApiKey::generate_new();

    socket
        .send(Message::Binary(new_refresh_token_bytes))
        .await
        .into_error(WebSocketError::Send)?;

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
        .into_error(WebSocketError::Send)?;

    loop {
        tokio::select! {
            result = socket.recv() => {
                match result {
                    Some(Err(_)) | None => break,
                    Some(Ok(_)) => continue,
                }
            }
            // TODO: event sending at some point?
        }
    }

    Ok(())
}
