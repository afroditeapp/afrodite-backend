//! Backup transfer routes for transferring data between clients
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
use model_chat::{BackupTransferInitialMessage, ClientRole};
use server_api::{S, app::GetAccessTokens};
use sha2::{Digest, Sha256};
use simple_backend::create_counters;

use super::super::utils::StatusCode;

mod source;
mod target;

use source::handle_source_client;
use target::handle_target_client;

pub const PATH_BACKUP_TRANSFER: &str = "/chat_api/backup_transfer";

/// 64 KiB
pub const MAX_BINARY_MESSAGE_SIZE: usize = 1024 * 64;

type Sha256Bytes = [u8; 32];

/// Transfer chat backup between clients using WebSocket.
///
/// This WebSocket connection facilitates secure backup transfer between two clients:
/// a target client (receiving data) and a source client (sending data).
///
/// Header `Sec-WebSocket-Protocol` must have `v1` as the first value.
///
/// ## Target Client Flow:
/// 1. Connect and send initial JSON message [BackupTransferInitialMessage] with [ClientRole::Target]
/// 2. Wait for source to connect (timeout: 1 hour). Sending empty
///    binary messages is possible to test connectivity.
/// 3. Receive byte count JSON message [model_chat::BackupTransferByteCount]
/// 4. Receive binary messages until all bytes transferred
///
/// ## Source Client Flow:
/// 1. Connect and send initial JSON message [BackupTransferInitialMessage] with
///    [ClientRole::Source] (must connect after target).
///    Note: Response has constant 1-second delay.
///    Connection closes if [BackupTransferInitialMessage::data_sha256] is invalid
///    or target is not connected.
/// 2. Receive data JSON message [BackupTransferData]
/// 3. Send byte count JSON message [model_chat::BackupTransferByteCount]
/// 4. Send binary messages containing the data until all bytes transferred.
///    Max size for a binary message is 64 KiB. Server will stop the data
///    transfer if binary message size is larger than the max size.
///
/// ## WebSocket Close Status Codes:
/// - 1000 (Normal Closure): Transfer completed successfully
/// - 1008 (Policy Violation): Yearly transfer budget exceeded
/// - No close status code: Other error
#[utoipa::path(
    get,
    path = PATH_BACKUP_TRANSFER,
    responses(
        (status = 101, description = "Switching protocols to WebSocket."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn get_backup_transfer(
    State(state): State<S>,
    websocket: WebSocketUpgrade,
    header_map: HeaderMap,
) -> std::result::Result<impl IntoResponse, StatusCode> {
    TRANSFER.get_backup_transfer.incr();

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

struct Disconnecter {
    _sender: tokio::sync::oneshot::Sender<()>,
    sha256: Sha256Bytes,
}

struct PendingConnections {
    /// Allow only single target connection per account.
    /// New connection replaces current one-shot channel, so old
    /// channel breaks and old connection will quit.
    target_connection_disconnecter: HashMap<AccountId, Disconnecter>,
    connections: HashMap<Sha256Bytes, PendingTransfer>,
}

impl PendingConnections {
    fn new() -> Self {
        Self {
            target_connection_disconnecter: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    fn exists(&self, sha256: &Sha256Bytes) -> bool {
        self.connections.contains_key(sha256)
    }

    fn replace_connection(
        &mut self,
        account_id: AccountId,
        sha256: Sha256Bytes,
        transfer: PendingTransfer,
    ) -> tokio::sync::oneshot::Receiver<()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        if let Some(existing_connection) = self.target_connection_disconnecter.insert(
            account_id,
            Disconnecter {
                _sender: sender,
                sha256,
            },
        ) {
            self.connections.remove(&existing_connection.sha256);
        }
        self.connections.insert(sha256, transfer);
        receiver
    }

    fn remove(&mut self, sha256: &Sha256Bytes) -> Option<PendingTransfer> {
        self.connections.remove(sha256)
    }
}

struct PendingTransfer {
    pub data: String,
    pub source_ready_tx: tokio::sync::oneshot::Sender<WebSocket>,
}

static PENDING_TRANSFERS: OnceLock<tokio::sync::RwLock<PendingConnections>> = OnceLock::new();

fn get_pending_transfers() -> &'static tokio::sync::RwLock<PendingConnections> {
    PENDING_TRANSFERS.get_or_init(|| tokio::sync::RwLock::new(PendingConnections::new()))
}

async fn handle_transfer_socket(mut socket: WebSocket, state: S) {
    let role_message = match tokio::time::timeout(Duration::from_secs(10), socket.recv()).await {
        Ok(Some(Ok(Message::Text(text)))) => text,
        _ => {
            TRANSFER.connection_error.incr();
            return;
        }
    };

    let initial_message: BackupTransferInitialMessage = match serde_json::from_str(&role_message) {
        Ok(msg) => msg,
        Err(_) => {
            TRANSFER.protocol_error.incr();
            return;
        }
    };

    match initial_message.role {
        ClientRole::Target => {
            let access_token = initial_message.access_token.unwrap_or_default();
            let data = initial_message.data.unwrap_or_default();

            if access_token.is_empty() || data.is_empty() {
                TRANSFER.protocol_error.incr();
                return;
            }

            let account_id = match state
                .access_token_exists(&model::AccessToken::new(access_token.clone()))
                .await
            {
                Some(id) => id,
                None => {
                    TRANSFER.invalid_access_token.incr();
                    return;
                }
            };

            TRANSFER.target_connected.incr();
            handle_target_client(
                state,
                socket,
                account_id,
                Sha256::digest(data.as_bytes()).into(),
                data,
            )
            .await;
        }
        ClientRole::Source => {
            let data_sha256 = initial_message.data_sha256.unwrap_or_default();

            if data_sha256.is_empty() {
                TRANSFER.protocol_error.incr();
                return;
            }

            // Parse hex SHA256
            let sha256: Sha256Bytes = match base16ct::lower::decode_vec(&data_sha256) {
                Ok(bytes) if bytes.len() == 32 => {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&bytes);
                    arr
                }
                _ => {
                    TRANSFER.protocol_error.incr();
                    return;
                }
            };

            TRANSFER.source_connected.incr();
            handle_source_client(socket, sha256).await;
        }
    }
}

create_counters!(
    TransferCounters,
    TRANSFER,
    CHAT_TRANSFER_COUNTERS_LIST,
    get_backup_transfer,
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
