
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

use std::net::SocketAddr;
use json_rpc::handle_json_rpc;
use manager_api::protocol::ClientConnectionReadWrite;
use manager_model::ManagerProtocolMode;

use manager_api::protocol::{ConnectionUtilsRead, ConnectionUtilsWrite};

use crate::server::app::S;

use error_stack::{Result, ResultExt};
use manager_model::ManagerProtocolVersion;

use super::utils::validate_api_key;

pub mod json_rpc;

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
}

pub async fn handle_connection_to_server<
    T: ClientConnectionReadWrite,
>(
    connection: T,
    address: SocketAddr,
    state: S,
) {
    match handle_connection_to_server_with_error(
        connection,
        address,
        state,
    ).await {
        Ok(()) => (),
        Err(e) => {
            let e = e.attach_printable(address);
            tracing::error!("{:?}", e);
        }
    }
}

async fn handle_connection_to_server_with_error<
    T: ClientConnectionReadWrite,
>(
    mut c: T,
    address: SocketAddr,
    state: S,
) -> Result<(), ServerError> {
    let version = c.receive_protocol_version()
        .await
        .change_context(ServerError::UnsupportedProtocolVersion)?;

    match version {
        ManagerProtocolVersion::V1 => (),
    }

    let api_key = c.receive_string_with_u32_len()
        .await
        .change_context(ServerError::ApiKey)?;

    if validate_api_key(&state, address, &api_key).is_err() {
        c.send_u8(0).await.change_context(ServerError::ApiKeyResponse)?;
        return Ok(());
    }

    c.send_u8(1).await.change_context(ServerError::ApiKeyResponse)?;

    let mode = c.receive_protocol_mode()
        .await
        .change_context(ServerError::UnsupportedProtocolMode)?;

    match mode {
        ManagerProtocolMode::JsonRpc => handle_json_rpc(c, address, state).await,
        ManagerProtocolMode::ListenServerEvents => todo!(),
    }
}
