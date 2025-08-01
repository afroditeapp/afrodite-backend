//! Manager protocol server code
//!
//! # Manager protocol
//!
//! 1. Client sends version byte.
//! 2. Client sends u32 little-endian API key length in bytes.
//! 3. Client sends UTF-8 API key.
//! 4. Server sends byte 1 if API key is correct. Byte 0 is sent and
//!    connection is closed when API key is incorrect.
//! 5. Client sends protocol mode byte.
//! 6. Next step is protocol mode specific.
//!
//! ## [manager_model::ManagerProtocolMode::JsonRpc]
//!
//! Client sends [manager_model::JsonRpcRequest] JSON and server sends
//! [manager_model::JsonRpcResponse] JSON.
//!
//! 1. Client sends u32 little-endian JSON length in bytes.
//! 2. Client sends UTF-8 JSON bytes.
//! 3. Server sends u32 little-endian JSON length in bytes.
//! 4. Server sends UTF-8 JSON bytes.
//! 5. Server closes the connection.
//!
//! ## [manager_model::ManagerProtocolMode::ListenServerEvents]
//!
//! Server sends [manager_model::ServerEvent] JSONs.
//!
//! 1. Server sends u32 little-endian JSON length in bytes.
//! 2. Server sends UTF-8 JSON bytes.
//! 3. Move to step 1.
//!
//! ## [manager_model::ManagerProtocolMode::JsonRpcLink]
//!
//! 1. Client sends u32 little-endian manager name length in bytes.
//! 2. Client sends UTF-8 manager name.
//! 3. Client sends u32 little-endian link password length in bytes.
//! 4. Client sends UTF-8 link password.
//! 5. Server sends byte 1 if login is correct. Byte 0 is sent and
//!    connection is closed when login is incorrect.
//!
//! Both server and client can send messages with the following format:
//!
//! - Message type (u8)
//! - Sequence number (u32 little-endian)
//! - JSON length (u32 little-endian)
//! - JSON data
//!
//! ### Message types
//!
//! #### [manager_model::JsonRpcLinkMessageType::Empty] (type 0)
//!
//! Used for connection keep-alive logic. Sequence number and JSON length are 0.
//!
//! ### [manager_model::JsonRpcLinkMessageType::ServerRequest] (type 1)
//!
//! Sequence numbers starts from 0 for new link connections
//! and the number will wrap. The number is incremented
//! after it is used.
//!
//! ### [manager_model::JsonRpcLinkMessageType::ServerResponse] (type 2)
//!
//! The response for [manager_model::JsonRpcLinkMessageType::ServerRequest].
//! The sequence number is the same as in the request.
//!
//! ## [manager_model::ManagerProtocolMode::BackupLink]
//!
//! 1. Client sends u32 little-endian link password length in bytes.
//! 2. Client sends UTF-8 link password.
//! 3. Server sends byte 1 if login is correct. Byte 0 is sent and
//!    connection is closed when login is incorrect.
//!
//! When two clients are connected, the traffic from one client is forwarded
//! to the another.
//!
//! The clients can send messages to each other with the following format:
//!
//! - Message type (u8)
//! - Sequence number (u32 little-endian)
//! - Data length (u32 little-endian)
//! - Data bytes

use std::net::SocketAddr;

use error_stack::{Result, ResultExt};
use json_rpc::handle_json_rpc;
use link::{backup::handle_backup_link, json_rpc::handle_json_rpc_link};
use manager_api::protocol::{
    ClientConnectionReadWrite, ClientConnectionWrite, ConnectionUtilsRead, ConnectionUtilsWrite,
};
use manager_model::{ManagerProtocolMode, ManagerProtocolVersion, ServerEvent};
use simple_backend_utils::ContextExt;
use tokio::io::AsyncReadExt;
use tracing::info;

use super::utils::validate_api_key;
use crate::server::app::S;

pub mod json_rpc;
pub mod link;

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Reading error")]
    Read,
    #[error("Writing error")]
    Write,
    #[error("Flush error")]
    Flush,
    #[error("Parsing error")]
    Parse,
    #[error("Serializing error")]
    Serialize,
    #[error("Unsupported protocol version")]
    UnsupportedProtocolVersion,
    #[error("Unsupported protocol mode")]
    UnsupportedProtocolMode,
    #[error("Unsupported string length")]
    UnsupportedStringLength,
    #[error("API key reading")]
    ApiKey,
    #[error("API key response")]
    ApiKeyResponse,
    #[error("JSON RPC request receiving failed")]
    JsonRpcRequestReceivingFailed,
    #[error("JSON RPC response sending failed")]
    JsonRpcResponseSendingFailed,
    #[error("Channel broken")]
    BrokenChannel,
    #[error("JSON RPC failed")]
    JsonRpcFailed,
    #[error("Client error")]
    Client,
    #[error("Server event channel is broken")]
    ServerEventChannelBroken,
    #[error("JSON RPC link related error")]
    JsonRpcLink,
    #[error("Backup link related error")]
    BackupLink,
}

pub async fn handle_connection_to_server<T: ClientConnectionReadWrite>(
    connection: T,
    address: SocketAddr,
    state: S,
) {
    match handle_connection_to_server_with_error(connection, address, state).await {
        Ok(()) => (),
        Err(e) => {
            let e = e.attach_printable(address);
            tracing::error!("{:?}", e);
        }
    }
}

async fn handle_connection_to_server_with_error<T: ClientConnectionReadWrite>(
    mut c: T,
    address: SocketAddr,
    state: S,
) -> Result<(), ServerError> {
    let version = c
        .receive_protocol_version()
        .await
        .change_context(ServerError::UnsupportedProtocolVersion)?;

    match version {
        ManagerProtocolVersion::V1 => (),
    }

    let api_key = c
        .receive_string_with_u32_len()
        .await
        .change_context(ServerError::ApiKey)?;

    if validate_api_key(&state, address, &api_key).is_err() {
        c.send_u8(0)
            .await
            .change_context(ServerError::ApiKeyResponse)?;
        return Ok(());
    }

    c.send_u8(1)
        .await
        .change_context(ServerError::ApiKeyResponse)?;

    let mode = c
        .receive_protocol_mode()
        .await
        .change_context(ServerError::UnsupportedProtocolMode)?;

    match mode {
        ManagerProtocolMode::JsonRpc => handle_json_rpc(c, address, state).await,
        ManagerProtocolMode::ListenServerEvents => handle_server_events(c, address, state).await,
        ManagerProtocolMode::JsonRpcLink => handle_json_rpc_link(c, address, state).await,
        ManagerProtocolMode::BackupLink => handle_backup_link(c, address, state).await,
    }
}

async fn handle_server_events<C: ClientConnectionReadWrite>(
    c: C,
    address: SocketAddr,
    state: S,
) -> Result<(), ServerError> {
    info!("Server events: {} connected", address);

    let (mut reader, writer) = tokio::io::split(c);

    let mut read_buffer = [0u8];
    let r = tokio::select! {
        _ = reader.read(&mut read_buffer) => {
            // Server disconnected
            Ok(())
        }
        r = send_server_events(writer, state) => {
            r
        },
    };

    info!("Server events: {} disconnected", address);
    r
}

async fn send_server_events<C: ClientConnectionWrite>(
    mut c: C,
    state: S,
) -> Result<(), ServerError> {
    let mut receiver = state.backend_events_receiver();

    loop {
        let events: Vec<ServerEvent> = receiver.borrow_and_update().clone();

        for e in events.iter() {
            c.send_server_event(e)
                .await
                .change_context(ServerError::Write)?;
        }

        if receiver.changed().await.is_err() {
            return Err(ServerError::ServerEventChannelBroken.report());
        }
    }
}
