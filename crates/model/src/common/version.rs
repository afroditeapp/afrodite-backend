use serde::{Deserialize, Serialize};
use simple_backend_model::VersionNumber;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum WebSocketClientTypeNumber {
    Android = 0,
    Ios = 1,
    Web = 2,
    /// Type number for test mode bots. It is the last available value
    /// to keep it more hidden.
    TestModeBot = 255,
}

impl TryFrom<u8> for WebSocketClientTypeNumber {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Android),
            1 => Ok(Self::Ios),
            2 => Ok(Self::Web),
            255 => Ok(Self::TestModeBot),
            _ => Err(format!("Unknown client type number {value}")),
        }
    }
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
