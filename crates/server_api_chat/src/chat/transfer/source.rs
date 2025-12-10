//! Source client handling for backup transfer

use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket};
use model_chat::BackupTransferTargetData;

use crate::chat::transfer::{Sha256Bytes, TRANSFER, get_pending_transfers};

pub async fn handle_source_client(socket: WebSocket, sha256: Sha256Bytes) {
    let wait_until = Instant::now() + Duration::from_secs(1);
    let result = handle_source_client_internal(socket, sha256).await;
    tokio::time::sleep_until(wait_until.into()).await;
    drop(result);
}

/// At least when SHA256 is invalid, socket must not be dropped before
/// sleep_until.
async fn handle_source_client_internal(
    mut socket: WebSocket,
    sha256: Sha256Bytes,
) -> Result<(), WebSocket> {
    let sha256_exists = { get_pending_transfers().read().await.exists(&sha256) };

    if !sha256_exists {
        return Err(socket);
    }

    let transfer = {
        match get_pending_transfers().write().await.remove(&sha256) {
            Some(transfer) => transfer,
            None => {
                TRANSFER.target_not_connected.incr();
                return Err(socket);
            }
        }
    };

    // Send data to source
    let data_message = BackupTransferTargetData {
        target_data: transfer.data,
    };
    let Ok(data_json) = serde_json::to_string(&data_message) else {
        TRANSFER.protocol_error.incr();
        return Err(socket);
    };

    if socket.send(Message::Text(data_json.into())).await.is_err() {
        TRANSFER.connection_error.incr();
        return Err(socket);
    }

    // Send source socket to target handler
    match transfer.source_ready_tx.send(socket) {
        Ok(()) => (),
        Err(socket) => {
            TRANSFER.connection_error.incr();
            return Err(socket);
        }
    }

    Ok(())
}
