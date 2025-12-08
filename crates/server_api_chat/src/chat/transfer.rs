//! Data transfer routes for transferring data between clients
//!

use std::{collections::HashMap, sync::OnceLock, time::Duration};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
};
use http::HeaderMap;
use model::AccountId;
use model_chat::{ClientRole, DataTransferInitialMessage, DataTransferPublicKey};
use server_api::S;
use simple_backend::create_counters;
use tokio::sync::Mutex;

use super::super::utils::StatusCode;

mod source;
mod target;

use source::handle_source_client;
use target::handle_target_client;

pub const PATH_TRANSFER_DATA: &str = "/chat_api/transfer_data";

/// 64 KiB
pub const MAX_BINARY_MESSAGE_SIZE: usize = 1024 * 64;

/// Transfer data between clients using WebSocket.
///
/// This WebSocket connection facilitates secure data transfer between two clients:
/// a target client (receiving data) and a source client (sending data).
///
/// Header `Sec-WebSocket-Protocol` must have `v1` as the first value.
///
/// ## Target Client Flow:
/// 1. Connect and send initial JSON message [DataTransferInitialMessage] with [ClientRole::Target]
/// 2. Wait for source to connect (timeout: 1 hour)
/// 3. Receive byte count JSON message [model_chat::DataTransferByteCount]
/// 4. Receive binary messages until all bytes transferred
///
/// ## Source Client Flow:
/// 1. Connect and send initial JSON message [DataTransferInitialMessage] with [ClientRole::Source]
///    (must connect after target).
///    Note: Response has constant 1-second delay. Connection closes if password is invalid
///    or target is not connected.
/// 2. Receive public key JSON message [DataTransferPublicKey]
/// 3. Send byte count JSON message [model_chat::DataTransferByteCount]
/// 4. Send binary messages containing the data until all bytes transferred.
///    Max size for a binary message is 64 KiB. Server will stop the data
///    transfer if binary message size is larger than the max size.
///
/// ## Transfer Budget Enforcement:
/// When the yearly transfer budget is exceeded, both WebSockets (source and target)
/// are closed with status code 4000.
#[utoipa::path(
    get,
    path = PATH_TRANSFER_DATA,
    responses(
        (status = 101, description = "Switching protocols to WebSocket."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn get_transfer_data(
    State(state): State<S>,
    websocket: WebSocketUpgrade,
    header_map: HeaderMap,
) -> std::result::Result<impl IntoResponse, StatusCode> {
    TRANSFER.get_transfer_data.incr();

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

    let response = websocket
        .protocols(["v1"])
        .on_upgrade(move |socket| handle_transfer_socket(socket, state));

    Ok(response)
}

type PendingConnections = Mutex<HashMap<AccountId, PendingTransfer>>;

struct PendingTransfer {
    pub password: String,
    pub source_ready_tx: Option<tokio::sync::oneshot::Sender<WebSocket>>,
}

static PENDING_TRANSFERS: OnceLock<PendingConnections> = OnceLock::new();

fn get_pending_transfers() -> &'static PendingConnections {
    PENDING_TRANSFERS.get_or_init(|| Mutex::new(HashMap::new()))
}

struct PendingTransfersManager;

impl PendingTransfersManager {
    async fn insert(account_id: impl Into<AccountId>, transfer: PendingTransfer) {
        get_pending_transfers()
            .lock()
            .await
            .insert(account_id.into(), transfer);
    }

    async fn remove(account_id: impl Into<AccountId>) -> Option<PendingTransfer> {
        get_pending_transfers()
            .lock()
            .await
            .remove(&account_id.into())
    }
}

async fn handle_transfer_socket(mut socket: WebSocket, state: S) {
    let role_message = match tokio::time::timeout(Duration::from_secs(10), socket.recv()).await {
        Ok(Some(Ok(Message::Text(text)))) => text,
        _ => {
            TRANSFER.connection_error.incr();
            return;
        }
    };

    let initial_message: DataTransferInitialMessage = match serde_json::from_str(&role_message) {
        Ok(msg) => msg,
        Err(_) => {
            TRANSFER.protocol_error.incr();
            return;
        }
    };

    match initial_message.role {
        ClientRole::Target => {
            let access_token = initial_message.access_token.unwrap_or_default();
            let public_key = initial_message.public_key.unwrap_or_default();
            let password = initial_message.password.unwrap_or_default();

            if access_token.is_empty() || public_key.is_empty() || password.is_empty() {
                TRANSFER.protocol_error.incr();
                return;
            }

            let first_message_to_source = DataTransferPublicKey { public_key };

            TRANSFER.target_connected.incr();
            handle_target_client(
                socket,
                state,
                access_token,
                first_message_to_source,
                password,
            )
            .await;
        }
        ClientRole::Source => {
            let account_id = initial_message.account_id.unwrap_or_default();
            let password = initial_message.password.unwrap_or_default();

            if account_id.is_empty() || password.is_empty() {
                TRANSFER.protocol_error.incr();
                return;
            }

            let Ok(account_id) = TryInto::try_into(account_id) else {
                TRANSFER.protocol_error.incr();
                return;
            };

            TRANSFER.source_connected.incr();
            handle_source_client(socket, account_id, password).await;
        }
    }
}

create_counters!(
    TransferCounters,
    TRANSFER,
    CHAT_TRANSFER_COUNTERS_LIST,
    get_transfer_data,
    target_connected,
    source_connected,
    connection_error,
    protocol_error,
    target_not_connected,
    invalid_password,
    invalid_access_token,
    timeout,
    transfer_completed,
    transfer_error,
    budget_exceeded,
);
