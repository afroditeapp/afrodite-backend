//! Common routes to all microservices
//!

use std::{net::SocketAddr, time::Duration};

use axum::{
    body::Bytes,
    extract::{
        ConnectInfo, Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use headers::ContentType;
use http::HeaderMap;
use model::{
    AccessToken, AccountIdInternal, BackendVersion, EventToClient, EventToClientInternal,
    PendingNotificationFlags, RefreshToken, SyncDataVersionFromClient, WebSocketClientTypeNumber,
};
use model_server_data::AuthPair;
use server_common::websocket::WebSocketError;
use server_data::{
    app::{BackendVersionProvider, EventManagerProvider, GetConfig},
    read::GetReadCommandsCommon,
    write::GetWriteCommandsCommon,
};
use server_state::{
    app::{
        AdminNotificationProvider, ApiUsageTrackerProvider, ClientVersionTrackerProvider,
        GetAccessTokens, IpAddressUsageTrackerProvider,
    },
    state_impl::{ReadData, WriteData},
};
use simple_backend::{
    app::FilePackageProvider,
    create_counters,
    perf::websocket::{self, ConnectionTracker},
    web_socket::WebSocketManager,
};
use simple_backend_utils::IntoReportFromString;
use tokio::time::Instant;
use tracing::{error, info};

use super::utils::StatusCode;
use crate::{
    S,
    result::{WrappedContextExt, WrappedResultExt},
    utils::Json,
};

mod client_config;
pub use client_config::*;

mod push_notification;
pub use push_notification::*;

pub const PATH_GET_VERSION: &str = "/common_api/version";

/// Get backend version.
#[utoipa::path(
    get,
    path = PATH_GET_VERSION,
    security(),
    responses(
        (status = 200, description = "Version information.", body = BackendVersion),
    )
)]
pub async fn get_version(State(state): State<S>) -> Json<BackendVersion> {
    COMMON.get_version.incr();
    state.backend_version().into()
}

// TODO(prod): HTTP cache header support for file package access

pub const PATH_FILE_PACKAGE_ACCESS: &str = "/{*path}";

pub async fn get_file_package_access(
    State(state): State<S>,
    Path(path_parts): Path<Vec<String>>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> Result<(TypedHeader<ContentType>, Bytes), StatusCode> {
    COMMON.get_file_package_access.incr();
    check_ip_allowlist(&state, address)?;
    let wanted_file = path_parts.join("/");
    let (content_type, data) = state
        .file_package()
        .data(&wanted_file)
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok((TypedHeader(content_type), data))
}

pub const PATH_FILE_PACKAGE_ACCESS_ROOT: &str = "/";

pub async fn get_file_package_access_root(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> Result<(TypedHeader<ContentType>, Bytes), StatusCode> {
    COMMON.get_file_package_access_root.incr();
    check_ip_allowlist(&state, address)?;
    let (content_type, data) = state
        .file_package()
        .data("index.html")
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok((TypedHeader(content_type), data))
}

fn check_ip_allowlist(state: &S, address: SocketAddr) -> Result<(), StatusCode> {
    if let Some(config) = state.config().simple_backend().file_package() {
        if config.disable_ip_allowlist || config.ip_allowlist.iter().any(|v| *v == address.ip()) {
            Ok(())
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub use utils::api::PATH_CONNECT;

// ------------------------- WebSocket -------------------------

// TODO(prod): Change WebSocket protocol version to 1?

/// Connect to server using WebSocket after getting refresh and access tokens.
/// Connection is required as API access is allowed for connected clients.
///
/// Protocol:
/// 1. Client sends version information as Binary message, where
///    - u8: Client WebSocket protocol version (currently 0).
///    - u8: Client type number. (0 = Android, 1 = iOS, 2 = Web, 255 = Test mode bot)
///    - u16: Client Major version.
///    - u16: Client Minor version.
///    - u16: Client Patch version.
///
///    The u16 values are in little endian byte order.
/// 2. Client sends current refresh token as Binary message.
/// 3. If server supports the client, the server sends next refresh token
///    as Binary message.
///    If server does not support the client, the server sends Text message
///    and closes the connection without WebSocket Close message.
/// 4. Server sends new access token as Binary message. The client must
///    convert the token to base64url encoding without padding.
///    (At this point API can be used.)
/// 5. Client sends list of current data sync versions as Binary message, where
///    items are [u8; 2] and the first u8 of an item is the data type number
///    and the second u8 of an item is the sync version number for that data.
///    If client does not have any version of the data, the client should
///    send 255 as the version number.
///
///    Available data types:
///    - 0: Account
/// 6. Server starts to send JSON events as Text messages and empty binary
///    messages to test connection to the client. Client can ignore the empty
///    binary messages.
/// 7. If needed, the client sends empty binary messages to test connection to
///    the server.
///
/// The new access token is valid until this WebSocket is closed or the
/// server detects a timeout. To prevent the timeout the client must
/// send a WebScoket ping message before 6 minutes elapses from connection
/// establishment or previous ping message.
///
/// `Sec-WebSocket-Protocol` header must have 2 protocols/values. The first
/// is "0" and that protocol is accepted. The second is access token of
/// currently logged in account. The token is base64url encoded without padding.
#[utoipa::path(
    get,
    path = PATH_CONNECT,
    responses(
        (status = 101, description = "Switching protocols."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn get_connect_websocket(
    State(state): State<S>,
    websocket: WebSocketUpgrade,
    header_map: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ws_manager: WebSocketManager,
) -> std::result::Result<impl IntoResponse, StatusCode> {
    COMMON.get_connect_websocket.incr();

    // NOTE: This handler does not have authentication layer enabled, so
    // authentication must be done manually.

    let mut protocols_iterator = header_map
        .get(http::header::SEC_WEBSOCKET_PROTOCOL)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_str()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .split(',')
        .map(|v| v.trim());

    if protocols_iterator.next() != Some("0") {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let id = if let Some(access_token) = protocols_iterator.next() {
        let access_token = AccessToken::new(access_token.to_string());
        state
            .access_token_exists(&access_token)
            .await
            .ok_or_else(|| {
                COMMON.websocket_access_token_not_found.incr();
                StatusCode::UNAUTHORIZED
            })?
    } else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    state
        .api_usage_tracker()
        .incr(id, |u| &u.get_connect_websocket)
        .await;
    state
        .ip_address_usage_tracker()
        .mark_ip_used(id, addr.ip())
        .await;

    info!("get_connect_websocket for '{}'", id.id.as_i64());

    let response = websocket
        .protocols(["0"])
        .on_upgrade(move |socket| handle_socket(socket, addr, id, state, ws_manager));
    Ok(response)
}

async fn handle_socket(
    socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: S,
    mut ws_manager: WebSocketManager,
) {
    // TODO(prod): Remove account details printing before 1.0

    info!(
        "handle_socket for '{}', address: {}",
        id.id.as_i64(),
        address
    );
    let quit_lock = if let Some(quit_lock) = ws_manager.get_ongoing_ws_connection_quit_lock().await
    {
        quit_lock
    } else {
        return;
    };

    tokio::select! {
        _ = ws_manager.server_quit_detected() => {
            info!("Server quit detected, closing WebSocket connection for '{}', address: {}", id.id.as_i64(), address);
            // It seems that this does not run when server closes.
            // handle_socket_result will return Ok(()) when that happens.
            let result = state.write(move |cmds| async move {
                cmds.common()
                    .end_connection_session(id, address)
                    .await
            }).await;

            if let Err(e) = result {
                error!("server quit end_connection_session, {e:?}, for '{}', address: {}", id.id.as_i64(), address);
            }
        },
        r = handle_socket_result(socket, address, id, &state) => {
            match r {
                Ok(()) => {
                    info!("handle_socket_result returned Ok for '{}', address: {}", id.id.as_i64(), address);
                    let result = state.write(move |cmds| async move {
                        cmds.common()
                            .end_connection_session(id, address)
                            .await
                    }).await;

                    if let Err(e) = result {
                        error!("end_connection_session, {e:?}, for '{}', address: {}", id.id.as_i64(), address);
                    }
                },
                Err(e) => {
                    error!("handle_socket_result returned Err {e:?} for '{}', address: {}", id.id.as_i64(), address);

                    let result = state.write(move |cmds| async move {
                        cmds.common().logout(id).await
                    }).await;

                    if let Err(e) = result {
                        error!("logout, {e:?}, for '{}', address: {}", id.id.as_i64(), address);
                    }
                }
            }
        }
    }

    state
        .event_manager()
        .trigger_push_notification_sending_check_if_needed(id)
        .await;

    drop(quit_lock);

    info!(
        "Connection for '{}' closed, address: {}",
        id.id.as_i64(),
        address
    );

    COMMON.websocket_disconnected.incr();
}

async fn handle_socket_result(
    mut socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: &S,
) -> crate::result::Result<(), WebSocketError> {
    info!(
        "handle_socket_result for '{}', address: {}",
        id.id.as_i64(),
        address
    );

    // Receive protocol version byte.
    let client_is_supported = match socket
        .recv()
        .await
        .ok_or(WebSocketError::Receive.report())?
        .change_context(WebSocketError::Receive)?
    {
        Message::Binary(version) => match version.to_vec().as_slice() {
            [0, info_bytes @ ..] => {
                let info = model::WebSocketClientInfo::parse(info_bytes)
                    .into_error_string(WebSocketError::ProtocolError)?;

                match info.client_type {
                    WebSocketClientTypeNumber::Android => {
                        COMMON.websocket_client_type_android.incr()
                    }
                    WebSocketClientTypeNumber::Ios => COMMON.websocket_client_type_ios.incr(),
                    WebSocketClientTypeNumber::Web => COMMON.websocket_client_type_web.incr(),
                    WebSocketClientTypeNumber::TestModeBot => {
                        COMMON.websocket_client_type_test_mode_bot.incr()
                    }
                }

                state
                    .client_version_tracker()
                    .track_version(info.client_version)
                    .await;

                if let Some(min_version) = state.config().min_client_version() {
                    min_version.received_version_is_accepted(info.client_version)
                } else {
                    true
                }
            }
            _ => return Err(WebSocketError::ProtocolError.report()),
        },
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
                COMMON.websocket_refresh_token_not_found.incr();
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
            .send(Message::Text(String::new().into()))
            .await
            .change_context(WebSocketError::Send)?;
        drop(socket);
        return Err(WebSocketError::ClientVersionUnsupported.report());
    }

    // Refresh check was successful, so the new refresh token can be sent.

    let (new_refresh_token, new_refresh_token_bytes) = RefreshToken::generate_new_with_bytes();
    let (new_access_token, new_access_token_bytes) = AccessToken::generate_new_with_bytes();

    socket
        .send(Message::Binary(new_refresh_token_bytes.into()))
        .await
        .change_context(WebSocketError::Send)?;

    let mut event_receiver = state
        .write(move |cmds| async move {
            // Prevent sending push notification if this connection
            // replaces the old connection.
            cmds.events()
                .remove_specific_pending_notification_flags_from_cache(
                    id,
                    PendingNotificationFlags::all(),
                )
                .await;
            // Create new event channel, so old one will break.
            // Also update tokens.
            cmds.common()
                .set_new_auth_pair(
                    id,
                    AuthPair {
                        access: new_access_token,
                        refresh: new_refresh_token,
                    },
                    Some(address),
                )
                .await
        })
        .await
        .change_context(WebSocketError::DatabaseSaveTokensOrOtherError)?
        .ok_or(WebSocketError::EventChannelCreationFailed.report())?;

    socket
        .send(Message::Binary(new_access_token_bytes.into()))
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

    state
        .data_all_access()
        .handle_new_websocket_connection(&mut socket, id, data_sync_versions)
        .await?;

    if state
        .admin_notification()
        .get_notification_state(id)
        .await
        .is_some()
    {
        send_event(&mut socket, EventToClientInternal::AdminNotification).await?;
    }

    // TODO(prod): Remove extra logging from this file.

    COMMON.websocket_connected.incr();
    let connection_trackers = WebSocketConnectionTrackers::new(state, id).await?;

    let mut timeout_timer = ConnectionPingTracker::new();

    loop {
        tokio::select! {
            result = socket.recv() => {
                match result {
                    Some(Err(_)) | None => break,
                    Some(Ok(value)) =>
                        match value {
                            Message::Binary(data) if data.is_empty() => {
                                // Client sent a ping message.
                                // Reset connection timeout to prevent server
                                // disconnecting this connection.
                                timeout_timer.reset().await;
                            },
                            Message::Ping(_) => {
                                // Client sent a ping message.
                                // Reset connection timeout to prevent server
                                // disconnecting this connection.
                                timeout_timer.reset().await;
                            },
                            Message::Pong(_) => (),
                            // TODO(prod): Remove data and address?
                            Message::Binary(data) => {
                                error!("Client sent unexpected binary message: {:?}, address: {}", data, address);
                            }
                            Message::Text(text) => {
                                error!("Client sent unexpected text message: {:?}, address: {}", text, address);
                            }
                            Message::Close(_) => break,
                        }
                }
            }
            event = event_receiver.recv() => {
                match event {
                    Some(internal_event) => {
                        let event: EventToClient = internal_event.to_client_event();
                        let event = serde_json::to_string(&event)
                            .change_context(WebSocketError::Serialize)?;
                        socket.send(Message::Text(event.into()))
                            .await
                            .change_context(WebSocketError::Send)?;
                        // If event is pending notification related, the cached
                        // pending notification flags are removed in the related
                        // HTTP route handlers using event manager assuming
                        // that client has received the event.
                    },
                    None => {
                        error!("Event receiver channel broken: id: {}, address: {}", id.id.as_i64(), address);
                        // New connection created another event receiver.
                        break;
                    },
                }
            }
            _ = timeout_timer.wait_timeout() => {
                // Connection timeout
                info!("Connection timeout for '{}', address: {}", id.id.as_i64(), address);
                break;
            }
        }
    }

    // Make sure that connection trackers are not dropped right
    // after those are created.
    drop(connection_trackers);

    Ok(())
}

struct ConnectionPingTracker {
    timer: tokio::time::Interval,
}

impl ConnectionPingTracker {
    const TIMEOUT_IN_SECONDS: u64 = 60 * 6;

    pub fn new() -> Self {
        let first_tick = Instant::now() + Duration::from_secs(Self::TIMEOUT_IN_SECONDS);
        Self {
            timer: tokio::time::interval_at(
                first_tick,
                Duration::from_secs(Self::TIMEOUT_IN_SECONDS),
            ),
        }
    }

    pub async fn wait_timeout(&mut self) {
        self.timer.tick().await;
    }

    pub async fn reset(&mut self) {
        self.timer.reset();
    }
}

struct WebSocketConnectionTrackers {
    _all: ConnectionTracker,
    _gender_specific: Option<ConnectionTracker>,
}

impl WebSocketConnectionTrackers {
    async fn new(state: &S, id: AccountIdInternal) -> crate::result::Result<Self, WebSocketError> {
        let info = state
            .read()
            .common()
            .bot_and_gender_info(id)
            .await
            .change_context(WebSocketError::DatabaseBotAndGenderInfoQuery)?;

        let all = if info.is_bot {
            websocket::BotConnections::create().into()
        } else {
            websocket::Connections::create().into()
        };

        let gender_specific = if info.is_bot {
            if info.gender.is_man() {
                Some(websocket::BotConnectionsMen::create().into())
            } else if info.gender.is_woman() {
                Some(websocket::BotConnectionsWomen::create().into())
            } else if info.gender.is_non_binary() {
                Some(websocket::BotConnectionsNonbinaries::create().into())
            } else {
                None
            }
        } else if info.gender.is_man() {
            Some(websocket::ConnectionsMen::create().into())
        } else if info.gender.is_woman() {
            Some(websocket::ConnectionsWomen::create().into())
        } else if info.gender.is_non_binary() {
            Some(websocket::ConnectionsNonbinaries::create().into())
        } else {
            None
        };

        Ok(Self {
            _all: all,
            _gender_specific: gender_specific,
        })
    }
}

async fn send_event(
    socket: &mut WebSocket,
    event: impl Into<EventToClient>,
) -> crate::result::Result<(), WebSocketError> {
    let event: EventToClient = event.into();
    let event = serde_json::to_string(&event).change_context(WebSocketError::Serialize)?;
    socket
        .send(Message::Text(event.into()))
        .await
        .change_context(WebSocketError::Send)?;

    Ok(())
}

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_COUNTERS_LIST,
    get_version,
    get_file_package_access,
    get_file_package_access_root,
    get_connect_websocket,
    websocket_access_token_not_found,
    websocket_refresh_token_not_found,
    websocket_connected,
    websocket_disconnected,
    websocket_client_type_android,
    websocket_client_type_ios,
    websocket_client_type_web,
    websocket_client_type_test_mode_bot,
);
