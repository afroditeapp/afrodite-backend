//! Target client handling for data transfer

use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use model::AccessToken;
use model_chat::{DataTransferByteCount, DataTransferPublicKey};
use server_api::{S, app::GetAccessTokens};

use super::{MAX_BINARY_MESSAGE_SIZE, PendingTransfer};
use crate::chat::transfer::{PendingTransfersManager, TRANSFER};

pub async fn handle_target_client(
    mut target_socket: WebSocket,
    state: S,
    access_token: String,
    first_message_to_source: DataTransferPublicKey,
    password: String,
) {
    let access_token = AccessToken::new(access_token);
    let account_id = match state.access_token_exists(&access_token).await {
        Some(id) => id.as_id(),
        None => {
            TRANSFER.invalid_access_token.incr();
            return;
        }
    };

    let (source_ready_tx, source_ready_rx) = tokio::sync::oneshot::channel();

    let transfer = PendingTransfer {
        password,
        source_ready_tx: Some(source_ready_tx),
    };

    PendingTransfersManager::insert(account_id, transfer).await;

    let timeout = Duration::from_secs(60 * 60);
    let mut source_socket = match tokio::time::timeout(timeout, source_ready_rx).await {
        Ok(Ok(source_socket)) => {
            PendingTransfersManager::remove(account_id).await;
            source_socket
        }
        Ok(Err(_)) | Err(_) => {
            TRANSFER.timeout.incr();
            PendingTransfersManager::remove(account_id).await;
            return;
        }
    };

    let Ok(first_message_to_source) = serde_json::to_string(&first_message_to_source) else {
        TRANSFER.protocol_error.incr();
        return;
    };

    if target_socket
        .send(Message::Text(first_message_to_source.into()))
        .await
        .is_err()
    {
        TRANSFER.connection_error.incr();
        return;
    }

    let byte_count_message = match source_socket.recv().await {
        Some(Ok(Message::Text(text))) => text,
        _ => {
            TRANSFER.connection_error.incr();
            return;
        }
    };

    let byte_count = match serde_json::from_str::<DataTransferByteCount>(&byte_count_message) {
        Ok(byte_count) => byte_count.byte_count,
        Err(_) => {
            TRANSFER.protocol_error.incr();
            return;
        }
    };

    if target_socket
        .send(Message::Text(byte_count_message.clone()))
        .await
        .is_err()
    {
        TRANSFER.connection_error.incr();
        return;
    }

    perform_transfer(&mut source_socket, &mut target_socket, byte_count).await;
    TRANSFER.transfer_completed.incr();
}

async fn perform_transfer(
    source_socket: &mut WebSocket,
    target_socket: &mut WebSocket,
    expected_bytes: u64,
) {
    let mut total_bytes_transferred = 0u64;

    loop {
        match source_socket.recv().await {
            Some(Ok(Message::Binary(data))) => {
                let data_len = data.len() as u64;

                if data.len() > MAX_BINARY_MESSAGE_SIZE {
                    TRANSFER.protocol_error.incr();
                    return;
                }

                total_bytes_transferred += data_len;

                if target_socket.send(Message::Binary(data)).await.is_err() {
                    TRANSFER.transfer_error.incr();
                    return;
                }

                if total_bytes_transferred >= expected_bytes {
                    return;
                }
            }
            Some(Ok(Message::Close(_))) | None => {
                return;
            }
            Some(Err(_)) => {
                TRANSFER.transfer_error.incr();
                return;
            }
            Some(Ok(Message::Text(_)))
            | Some(Ok(Message::Ping(_)))
            | Some(Ok(Message::Pong(_))) => continue,
        }
    }
}
