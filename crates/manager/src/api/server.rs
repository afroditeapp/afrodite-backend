
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
use manager_model::JsonRpcRequest;
use manager_model::JsonRpcResponse;
use manager_model::ManagerProtocolMode;
use manager_model::ServerEvent;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use crate::server::app::S;

use error_stack::{Result, ResultExt};
use manager_model::ManagerProtocolVersion;

use super::utils::validate_api_key;

pub mod json_rpc;

pub trait ClientConnectionReadWrite: ClientConnectionRead + ClientConnectionWrite {}
impl <T: ClientConnectionRead + ClientConnectionWrite> ClientConnectionReadWrite for T {}

pub trait ClientConnectionRead: tokio::io::AsyncRead + Send + std::marker::Unpin + 'static {}
impl <T: tokio::io::AsyncRead + Send + std::marker::Unpin + 'static> ClientConnectionRead for T {}

pub trait ClientConnectionWrite: tokio::io::AsyncWrite + Send + std::marker::Unpin + 'static {}
impl <T: tokio::io::AsyncWrite + Send + std::marker::Unpin + 'static> ClientConnectionWrite for T {}

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


pub(crate) trait ConnectionUtilsRead: tokio::io::AsyncRead + Unpin  {
    async fn receive_u8(&mut self) -> Result<u8, ServerError> {
        self.read_u8().await.change_context(ServerError::Read)
    }

    async fn receive_string_with_u32_len(&mut self) -> Result<String, ServerError> {
        let len = self.read_u32_le().await.change_context(ServerError::Read)?;
        let len_usize: usize = TryInto::<usize>::try_into(len).change_context(ServerError::UnsupportedStringLength)?;
        let mut vec: Vec<u8> = vec![0; len_usize];
        self.read_exact(&mut vec).await.change_context(ServerError::Read)?;
        String::from_utf8(vec).change_context(ServerError::Parse)
    }

    async fn receive_protocol_version(&mut self) -> Result<ManagerProtocolVersion, ServerError> {
        TryInto::<ManagerProtocolVersion>::try_into(self.receive_u8().await?)
            .change_context(ServerError::Parse)
    }

    async fn receive_protocol_mode(&mut self) -> Result<ManagerProtocolMode, ServerError> {
        TryInto::<ManagerProtocolMode>::try_into(self.receive_u8().await?)
            .change_context(ServerError::Parse)
    }

    async fn receive_json_rpc_request(&mut self) -> Result<JsonRpcRequest, ServerError> {
        let s = self.receive_string_with_u32_len().await
            .change_context(ServerError::Read)?;
        serde_json::from_str(&s)
            .change_context(ServerError::Parse)
    }

    async fn receive_json_rpc_response(&mut self) -> Result<JsonRpcResponse, ServerError> {
        let s = self.receive_string_with_u32_len().await
            .change_context(ServerError::Read)?;
        serde_json::from_str(&s)
            .change_context(ServerError::Parse)
    }

    async fn receive_server_event(&mut self) -> Result<ServerEvent, ServerError> {
        let s = self.receive_string_with_u32_len().await
            .change_context(ServerError::Read)?;
        serde_json::from_str(&s)
            .change_context(ServerError::Parse)
    }
}

impl <T: tokio::io::AsyncRead + Unpin> ConnectionUtilsRead for T {}

pub(crate) trait ConnectionUtilsWrite: tokio::io::AsyncWrite + Unpin  {
    async fn send_u8(&mut self, byte: u8) -> Result<(), ServerError> {
        self.write_u8(byte).await.change_context(ServerError::Write)?;
        self.flush().await.change_context(ServerError::Flush)?;
        Ok(())
    }

    async fn send_string_with_u32_len(&mut self, value: String) -> Result<(), ServerError> {
        let len_u32: u32 = TryInto::<u32>::try_into(value.len())
            .change_context(ServerError::UnsupportedStringLength)?;
        self.write_u32_le(len_u32)
            .await
            .change_context(ServerError::Write)?;
        self.write_all(value.as_bytes()).await.change_context(ServerError::Write)?;
        self.flush().await.change_context(ServerError::Flush)?;
        Ok(())
    }

    async fn send_json_rpc_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Result<(), ServerError> {
        let text = serde_json::to_string(&request)
            .change_context(ServerError::Serialize)?;
        self.send_string_with_u32_len(text).await
            .change_context(ServerError::Write)?;
        self.flush().await.change_context(ServerError::Flush)?;
        Ok(())
    }

    async fn send_json_rpc_response(
        &mut self,
        response: JsonRpcResponse
    ) -> Result<(), ServerError> {
        let text = serde_json::to_string(&response)
            .change_context(ServerError::Serialize)?;
        self.send_string_with_u32_len(text).await
            .change_context(ServerError::Write)?;
        self.flush().await.change_context(ServerError::Flush)?;
        Ok(())
    }
}

impl <T: tokio::io::AsyncWrite + Unpin> ConnectionUtilsWrite for T {}
