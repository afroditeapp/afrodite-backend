//! Target client handling for data transfer

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::Duration,
};

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use database_chat::current::read::chat::transfer::TransferBudgetCheckResult;
use model::AccessToken;
use model_chat::{DataTransferByteCount, DataTransferPublicKey};
use server_api::{
    S,
    app::{GetAccessTokens, GetConfig},
    db_write_raw,
};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use tokio::sync::Mutex;

use super::{MAX_BINARY_MESSAGE_SIZE, PendingTransfer};
use crate::{
    app::{ReadData, WriteData},
    chat::transfer::{PendingTransfersManager, TRANSFER},
};

type TransferLocks = Mutex<HashMap<model::AccountId, Arc<Mutex<()>>>>;

static ACCOUNT_TRANSFER_LOCKS: OnceLock<TransferLocks> = OnceLock::new();

fn get_account_transfer_locks() -> &'static TransferLocks {
    ACCOUNT_TRANSFER_LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

async fn get_account_lock(account_id: model::AccountId) -> Arc<Mutex<()>> {
    let mut locks = get_account_transfer_locks().lock().await;
    locks
        .entry(account_id)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

pub async fn handle_target_client(
    mut target_socket: WebSocket,
    state: S,
    access_token: String,
    first_message_to_source: DataTransferPublicKey,
    password: String,
) {
    let access_token = AccessToken::new(access_token);
    let account_id = match state.access_token_exists(&access_token).await {
        Some(id) => id,
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

    // Get account-specific lock to ensure only one transfer budget check happens at a time
    let account_lock = get_account_lock(account_id.into()).await;
    let guard = account_lock.lock().await;

    let yearly_limit = state
        .config()
        .limits_chat()
        .data_transfer_yearly_max_bytes
        .bytes();

    let budget_check_result = state
        .read()
        .chat()
        .transfer()
        .check_transfer_budget(account_id, byte_count, yearly_limit)
        .await;

    match budget_check_result {
        Ok(TransferBudgetCheckResult::Ok) => (),
        Ok(TransferBudgetCheckResult::ExceedsLimit) => {
            TRANSFER.budget_exceeded.incr();
            let close_frame = CloseFrame {
                code: 4000,
                reason: "Transfer budget exceeded".into(),
            };
            let _ = source_socket
                .send(Message::Close(Some(close_frame.clone())))
                .await;
            let _ = target_socket.send(Message::Close(Some(close_frame))).await;
            return;
        }
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

    let total_bytes_transferred =
        perform_transfer(&mut source_socket, &mut target_socket, byte_count.into()).await;

    if total_bytes_transferred > 0 {
        // Update the budget with actual bytes transferred
        let update_result = db_write_raw!(state, move |cmds| {
            cmds.chat()
                .transfer()
                .update_transfer_budget(account_id, total_bytes_transferred, yearly_limit)
                .await
        })
        .await;

        if update_result.is_ok() && total_bytes_transferred == Into::<i64>::into(byte_count) {
            TRANSFER.transfer_completed.incr();
        } else if update_result.is_err() {
            TRANSFER.protocol_error.incr();
        } else {
            TRANSFER.transfer_error.incr();
        }
    } else {
        TRANSFER.transfer_error.incr();
    }

    drop(guard);
}

async fn perform_transfer(
    source_socket: &mut WebSocket,
    target_socket: &mut WebSocket,
    expected_bytes: i64,
) -> i64 {
    let mut total_bytes_transferred = 0;

    loop {
        match source_socket.recv().await {
            Some(Ok(Message::Binary(data))) => {
                if data.len() > MAX_BINARY_MESSAGE_SIZE {
                    TRANSFER.protocol_error.incr();
                    return total_bytes_transferred;
                }

                total_bytes_transferred += data.len() as i64;

                if target_socket.send(Message::Binary(data)).await.is_err() {
                    TRANSFER.transfer_error.incr();
                    return total_bytes_transferred;
                }

                if total_bytes_transferred >= expected_bytes {
                    return total_bytes_transferred;
                }
            }
            Some(Ok(Message::Close(_))) | None => {
                return total_bytes_transferred;
            }
            Some(Err(_)) => {
                TRANSFER.transfer_error.incr();
                return total_bytes_transferred;
            }
            Some(Ok(Message::Text(_)))
            | Some(Ok(Message::Ping(_)))
            | Some(Ok(Message::Pong(_))) => continue,
        }
    }
}
