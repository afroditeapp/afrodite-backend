use serde::{Deserialize, Serialize};
use simple_backend_model::VersionNumber;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum WebSocketClientTypeNumber {
    Android = 0,
    Ios = 1,
    Web = 2,
    Bot = 3,
}

/// Parsed info from first WebSocket Binary message from client without
/// the protocol version byte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebSocketClientInfo {
    pub client_type: WebSocketClientTypeNumber,
    pub client_version: ClientVersion,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash, ToSchema)]
pub struct ClientVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ClientVersion {
    pub const BOT_CLIENT_VERSION: Self = Self {
        major: 0,
        minor: 0,
        patch: 0,
    };
}

impl From<ClientVersion> for VersionNumber {
    fn from(value: ClientVersion) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}
