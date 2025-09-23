//! Common routes
//!

use std::{net::SocketAddr, time::Duration};

use axum::{
    body::Bytes,
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use http::HeaderMap;
use model::{
    AccessToken, AccountIdInternal, BackendVersion, ClientVersion, EventToClient,
    EventToClientInternal, RefreshToken, SyncDataVersionFromClient, WebSocketClientInfo,
    WebSocketClientTypeNumber,
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
    create_counters,
    perf::websocket::{self, ConnectionTracker},
    web_socket::WebSocketManager,
};
use simple_backend_utils::{IntoReportFromString, time::DurationValue};
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

mod data_export;
pub use data_export::*;

mod file_package;
pub use file_package::*;

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

pub use utils::api::PATH_CONNECT;

// ------------------------- WebSocket -------------------------

/// Connect to server using WebSocket after getting refresh and access tokens.
/// Connection is required as API access is allowed for connected clients.
///
/// Protocol:
/// 1. Server sends one of these byte values as Binary message:
///    - 0, continue to data sync, move to step 5, at this point API can be used.
///    - 1, access token and refresh token refresh is needed, move to step 2.
///    - 2, unsupported client version, server closes the connection
///      without sending WebSocket Close message.
///    - 3, invalid access token, server closes the connection
///      without sending WebSocket Close message.
/// 2. Client sends current refresh token as Binary message.
/// 3. Server sends new refresh token as Binary message.
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
/// `Sec-WebSocket-Protocol` header must have the following values:
///   - Client WebSocket protocol version string (currently "v1").
///   - Client access token string (prefix 't' and base64url encoded token
///     without base64url padding).
///   - Client info string (prefix 'c' and values separated with '_' character)
///     - Client type number (0 = Android, 1 = iOS, 2 = Web, 255 = Test mode bot).
///     - Client major version number.
///     - Client minor version number.
///     - Client patch version number.
#[utoipa::path(
    get, // or CONNECT method if using HTTP/2
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

    if protocols_iterator.next() != Some("v1") {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let access_token = protocols_iterator
        .next()
        .and_then(|v| v.strip_prefix("t"))
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
        .map(|v| AccessToken::new(v.to_string()))?;

    let info = protocols_iterator
        .next()
        .and_then(|v| v.strip_prefix("c"))
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
        .and_then(|v| {
            let mut iterator = v.split('_');

            let client_type = iterator
                .next()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
                .and_then(|v| str::parse::<u8>(v).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR))
                .and_then(|v| {
                    TryInto::<WebSocketClientTypeNumber>::try_into(v)
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                })?;

            let major = iterator
                .next()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
                .and_then(|v| {
                    str::parse::<u16>(v).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                })?;

            let minor = iterator
                .next()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
                .and_then(|v| {
                    str::parse::<u16>(v).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                })?;

            let patch = iterator
                .next()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
                .and_then(|v| {
                    str::parse::<u16>(v).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                })?;

            Ok(WebSocketClientInfo {
                client_type,
                client_version: ClientVersion {
                    major,
                    minor,
                    patch,
                },
            })
        })?;

    let id = state.access_token_exists(&access_token).await;

    if let Some(id) = &id {
        state
            .api_usage_tracker()
            .incr(id.id, |u| &u.get_connect_websocket)
            .await;
        state
            .ip_address_usage_tracker()
            .mark_ip_used(id.id, addr.ip())
            .await;
    } else {
        COMMON.websocket_access_token_not_found.incr();
    }

    let response = websocket.protocols(["v1"]).on_upgrade(move |socket| {
        handle_socket_basic_errors(socket, addr, id, state, ws_manager, info)
    });
    Ok(response)
}

async fn handle_socket_basic_errors(
    mut socket: WebSocket,
    address: SocketAddr,
    id: Option<AccountIdInternal>,
    state: S,
    ws_manager: WebSocketManager,
    info: WebSocketClientInfo,
) {
    if let Some(id) = id {
        let is_supported_client = {
            match info.client_type {
                WebSocketClientTypeNumber::Android => COMMON.websocket_client_type_android.incr(),
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

            if info.client_type == WebSocketClientTypeNumber::TestModeBot {
                info.client_version == ClientVersion::BOT_CLIENT_VERSION
            } else if let Some(min_version) = state.config().min_client_version() {
                min_version.received_version_is_accepted(info.client_version)
            } else {
                true
            }
        };
        if is_supported_client {
            handle_socket(socket, address, id, state, ws_manager).await
        } else {
            let _ = socket.send(Message::Binary(Bytes::from_static(&[2]))).await;
        }
    } else {
        let _ = socket.send(Message::Binary(Bytes::from_static(&[3]))).await;
    }
}

async fn handle_socket(
    socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: S,
    mut ws_manager: WebSocketManager,
) {
    if state.config().general().debug_websocket_logging {
        info!(
            "handle_socket for '{}', address: {}",
            id.id.as_i64(),
            address
        );
    }

    let quit_lock = if let Some(quit_lock) = ws_manager.get_ongoing_ws_connection_quit_lock().await
    {
        quit_lock
    } else {
        return;
    };

    tokio::select! {
        _ = ws_manager.server_quit_detected() => {
            // It seems that this does not run when server closes.
            // handle_socket_result will return Ok(()) when that happens.
            let result = state
                .read()
                .cache_read_write_access()
                .websocket_cache_cmds()
                .delete_connection(id.into(), address)
                .await;

            if let Err(e) = result {
                error!("delete_connection failed, {e:?}");
            }
        },
        r = handle_socket_result(socket, address, id, &state) => {
            match r {
                Ok(()) => {
                    let result = state
                        .read()
                        .cache_read_write_access()
                        .websocket_cache_cmds()
                        .delete_connection(id.into(), address)
                        .await;

                    if let Err(e) = result {
                        error!("delete_connection failed, {e:?}");
                    }
                },
                Err(e) => {
                    if state.config().general().debug_websocket_logging {
                        error!("handle_socket_result returned error {e:?} for '{}', address: {}", id.id.as_i64(), address);
                    }

                    let result = state.write(move |cmds| async move {
                        cmds.common().logout(id).await
                    }).await;

                    if let Err(e) = result {
                        error!("logout failed, {e:?}");
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

    if state.config().general().debug_websocket_logging {
        info!(
            "Connection for '{}' closed, address: {}",
            id.id.as_i64(),
            address
        );
    }

    COMMON.websocket_disconnected.incr();
}

async fn handle_socket_result(
    mut socket: WebSocket,
    address: SocketAddr,
    id: AccountIdInternal,
    state: &S,
) -> crate::result::Result<(), WebSocketError> {
    let access_token_too_old = state
        .read()
        .common()
        .account_access_token_creation_time_from_cache(id)
        .await
        .change_context(WebSocketError::DatabaseAccessTokenCreationTime)?
        .map(|created| {
            created
                .ut
                .duration_value_elapsed(DurationValue::from_days(1))
        })
        .unwrap_or(true);

    let access_token_ip_address_changed = state
        .read()
        .common()
        .account_access_token_ip_address_from_cache(id)
        .await
        .change_context(WebSocketError::DatabaseAccessTokenIpAddress)?
        .map(|token_ip_address| token_ip_address.to_ip_addr() != address.ip())
        .unwrap_or(true);

    let mut event_receiver = if access_token_too_old || access_token_ip_address_changed {
        socket
            .send(Message::Binary(Bytes::from_static(&[1])))
            .await
            .change_context(WebSocketError::Send)?;

        let current_refresh_token = state
            .read()
            .common()
            .account_refresh_token_from_cache(id)
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

        // Refresh check was successful, so the new refresh token can be sent.

        let (new_refresh_token, new_refresh_token_bytes) = RefreshToken::generate_new_with_bytes();
        let (new_access_token, new_access_token_bytes) = AccessToken::generate_new_with_bytes();

        socket
            .send(Message::Binary(new_refresh_token_bytes.into()))
            .await
            .change_context(WebSocketError::Send)?;

        let event_receiver = state
            .read()
            .cache_read_write_access()
            .websocket_cache_cmds()
            .init_login_session(
                id.into(),
                AuthPair {
                    access: new_access_token,
                    refresh: new_refresh_token,
                },
                address,
                true,
            )
            .await
            .change_context(WebSocketError::DatabaseSaveTokensOrOtherError)?
            .ok_or(WebSocketError::EventChannelCreationFailed.report())?;

        socket
            .send(Message::Binary(new_access_token_bytes.into()))
            .await
            .change_context(WebSocketError::Send)?;

        event_receiver
    } else {
        let event_receiver = state
            .read()
            .cache_read_write_access()
            .websocket_cache_cmds()
            .init_login_session_using_existing_tokens(id.into(), address)
            .await
            .change_context(WebSocketError::EventChannelCreationFailed)?;

        socket
            .send(Message::Binary(Bytes::from_static(&[0])))
            .await
            .change_context(WebSocketError::Send)?;

        event_receiver
    };

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
        .get_unreceived_notification(id)
        .await
        .is_some()
    {
        send_event(&mut socket, EventToClientInternal::AdminNotification).await?;
    }

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
                            Message::Pong(_) |
                            Message::Binary(_) |
                            Message::Text(_) => (),
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
                        // New connection created another event receiver.
                        if state.config().general().debug_websocket_logging {
                            error!("Event receiver channel broken: id: {}, address: {}", id.id.as_i64(), address);
                        }
                        break;
                    },
                }
            }
            _ = timeout_timer.wait_timeout() => {
                // Connection timeout
                if state.config().general().debug_websocket_logging {
                    info!("Connection timeout for '{}', address: {}", id.id.as_i64(), address);
                }
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
