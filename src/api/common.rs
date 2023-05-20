//! Common routes to all microservices


// TODO: add app version route


use std::{net::SocketAddr, task::{self, ready, Poll}};

use axum::{Json, TypedHeader, extract::{WebSocketUpgrade, ConnectInfo, ws::{WebSocket, Message}}, response::IntoResponse, BoxError};

use bytes::BytesMut;
use hyper::{StatusCode, client::connect::{Connection, Connected}, server::accept::Accept};
use serde::{Deserialize, Serialize};
use tokio::{io::{DuplexStream, AsyncReadExt, AsyncWriteExt, AsyncWrite, AsyncRead}, sync::mpsc};
use utoipa::ToSchema;

use crate::server::app::AppState;

use super::{GetConfig, GetInternalApi, utils::{validate_sign_in_with_google_token, validate_sign_in_with_apple_token}, model::{AccountIdLight, AccountIdInternal}};

use tracing::error;

use super::{utils::ApiKeyHeader, GetApiKeys, GetUsers, ReadDatabase, WriteDatabase};


pub const PATH_CONNECT: &str = "/common_api/connect";

/// Connect to server using WebSocket after getting API key and two factor
/// connection token.
///
/// After WebSocket is connected the client should send the two factor
/// connection token to the WebSocket as text/string. The server will response
/// with text/string "ok" if the token is valid. If invalid token is detected,
/// then the server ends the connection and invalidates the API key (user needs
/// to log in again).
///
/// After successfull token validation. The client can use binary channel for
/// HTTP requests. Events from server are informed as JSON texts.
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
pub async fn get_connect_websocket(
    websocket: WebSocketUpgrade,
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: AppState,
) -> Result<impl IntoResponse, StatusCode> {
    let id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(websocket.on_upgrade(move |socket| handle_socket(socket, addr, id, state)))
}

async fn handle_socket(
    mut socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: AppState,
) {
    // TODO: add close server notification select? Or probably not needed as
    // server should shutdown after main future?

    // Connection token protocol check.
    let mut socket = match socket.recv().await {
        Some(Ok(Message::Text(msg))) => {
            if msg == "token" { // TODO: get token from database
                match socket.send(Message::Text("ok".to_string())).await {
                    Ok(()) => socket,
                    Err(e) => {
                        error!("{:?}", e);
                        return;
                    }
                }
            } else {
                // TODO: invalidate API key
                return
            }
        },
        Some(Err(e)) => {
            error!("{:?}", e);
            return;
        }
        Some(Ok(_)) | None => return,
    };

    // TODO: enable account connected flag for the account?

    let mut data = BytesMut::new();
    let (mut ws_side, axum_side) = tokio::io::duplex(512);
    match state.ws_http_sender.send(axum_side).await {
        Ok(()) => (),
        Err(e) => {
            error!("{:?}", e);
            return;
        }
    }

    loop {
        tokio::select! {
            result = ws_side.read_buf(&mut data) => {
                match result {
                    Ok(_) => {
                        match socket.send(Message::Binary(data.to_vec())).await {
                            Err(e) => {
                                error!("{:?}", e);
                                break;
                            }
                            Ok(()) => (),
                        }
                        data.clear();
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        break;
                    }
                }
            }
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Binary(data))) => {
                        match ws_side.write_all(&data).await {
                            Ok(()) => (),
                            Err(e) => {
                                error!("{:?}", e);
                                break;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        error!("{:?}", e);
                        break;
                    }
                    Some(Ok(_)) | None => break,
                }
            }
        }
    }

    // TODO: clear account connected flag?
}


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum EventToClient {
    AccountStateChanged,
}

// struct WsHttpConnection {
//     stream: DuplexStream,
// }

// impl AsyncWrite for WsHttpConnection {
//     fn poll_write(
//             mut self: std::pin::Pin<&mut Self>,
//             cx: &mut std::task::Context<'_>,
//             buf: &[u8],
//         ) -> std::task::Poll<Result<usize, std::io::Error>> {
//         std::pin::Pin::new(&mut self.stream).poll_write(cx, buf)
//     }
//     fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
//         std::pin::Pin::new(&mut self.stream).poll_flush(cx)
//     }

//     fn poll_shutdown(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
//         std::pin::Pin::new(&mut self.stream).poll_shutdown(cx)
//     }
// }

// impl AsyncRead for WsHttpConnection {
//     fn poll_read(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//         buf: &mut tokio::io::ReadBuf<'_>,
//     ) -> std::task::Poll<std::io::Result<()>> {
//         std::pin::Pin::new(&mut self.stream).poll_read(cx, buf)
//     }
// }

// impl Connection for WsHttpConnection {
//     fn connected(&self) -> hyper::client::connect::Connected {
//         Connected::new()
//     }
// }
