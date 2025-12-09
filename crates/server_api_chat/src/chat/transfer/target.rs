//! Target client handling for data transfer

use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket};
use database_chat::current::read::chat::transfer::TransferBudgetCheckResult;
use model_chat::DataTransferByteCount;
use server_api::{S, app::GetConfig, db_write_raw};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use tokio::time::Instant;

use super::{MAX_BINARY_MESSAGE_SIZE, PendingTransfer, Sha256Bytes};
use crate::{
    app::{ReadData, WriteData},
    chat::transfer::{TRANSFER, get_pending_transfers},
};

pub async fn handle_target_client(
    state: S,
    target_socket: WebSocket,
    account_id: model_chat::AccountIdInternal,
    sha256: Sha256Bytes,
    data: String,
) {
    let (source_ready_tx, source_ready_rx) = tokio::sync::oneshot::channel();

    let transfer = PendingTransfer {
        data,
        source_ready_tx,
    };

    let mut transfers = get_pending_transfers().write().await;
    let receiver = transfers.replace_connection(account_id.into(), sha256, transfer);
    drop(transfers);

    tokio::select! {
        _ = receiver => (),
        _ = handle_target_client_internal(state, target_socket, account_id, source_ready_rx) => (),
    }
}

async fn handle_target_client_internal(
    state: S,
    mut target_socket: WebSocket,
    account_id: model_chat::AccountIdInternal,
    mut source_ready_rx: tokio::sync::oneshot::Receiver<WebSocket>,
) {
    let wait_until = Instant::now() + Duration::from_secs(60 * 60);
    let mut source_socket = loop {
        tokio::select! {
            _ = tokio::time::sleep_until(wait_until) => {
                TRANSFER.timeout.incr();
                return;
            }
            r = &mut source_ready_rx => {
                match r {
                    Ok(ws) => break ws,
                    Err(_) => {
                        TRANSFER.connection_error.incr();
                        return;
                    }
                }
            }
            received_value = target_socket.recv() => {
                match received_value {
                    None | Some(Err(_)) => {
                        TRANSFER.connection_error.incr();
                        return;
                    }
                    Some(Ok(Message::Ping(_)))
                    | Some(Ok(Message::Pong(_))) => (),
                    Some(Ok(Message::Binary(data))) if data.is_empty() => (),
                    Some(Ok(Message::Close(_)))
                    | Some(Ok(Message::Binary(_)))
                    | Some(Ok(Message::Text(_))) => {
                    },

                }
            }
        }
    };

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
                code: 1008,
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
            let close_frame = CloseFrame {
                code: 1000,
                reason: "Transfer completed".into(),
            };
            let _ = source_socket
                .send(Message::Close(Some(close_frame.clone())))
                .await;
            let _ = target_socket.send(Message::Close(Some(close_frame))).await;
        } else if update_result.is_err() {
            TRANSFER.protocol_error.incr();
        } else {
            TRANSFER.transfer_error.incr();
        }
    } else {
        TRANSFER.transfer_error.incr();
    }
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
