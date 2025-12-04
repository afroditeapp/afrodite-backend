//! Source client handling for data transfer

use std::time::{Duration, Instant};

use axum::extract::ws::WebSocket;
use model::AccountId;

use crate::chat::transfer::{PendingTransfersManager, TRANSFER};

pub async fn handle_source_client(socket: WebSocket, account_id: AccountId, password: String) {
    let wait_until = Instant::now() + Duration::from_secs(1);
    let result = handle_source_client_internal(socket, account_id, password).await;
    tokio::time::sleep_until(wait_until.into()).await;
    drop(result);
}

async fn handle_source_client_internal(
    socket: WebSocket,
    account_id: AccountId,
    password: String,
) -> Result<(), WebSocket> {
    let Some(mut transfer) = PendingTransfersManager::remove(account_id).await else {
        TRANSFER.target_not_connected.incr();
        return Err(socket);
    };

    if transfer.password != password {
        TRANSFER.invalid_password.incr();
        return Err(socket);
    }

    if let Some(tx) = transfer.source_ready_tx.take() {
        let _ = tx.send(socket);
    }

    Ok(())
}
