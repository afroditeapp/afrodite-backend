use serde::{Deserialize, Serialize};
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
            _ => Err(format!("Unknown client type number {}", value)),
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

impl WebSocketClientInfo {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        match bytes {
            [client_type0, major0, major1, minor0, minor1, patch0, patch1] => {
                let client_type = WebSocketClientTypeNumber::try_from(*client_type0)?;
                let major = u16::from_le_bytes([*major0, *major1]);
                let minor = u16::from_le_bytes([*minor0, *minor1]);
                let patch = u16::from_le_bytes([*patch0, *patch1]);
                Ok(Self {
                    client_type,
                    client_version: ClientVersion {
                        major,
                        minor,
                        patch,
                    },
                })
            }
            _ => Err(format!("Invalid input byte count {}", bytes.len())),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash, ToSchema)]
pub struct ClientVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}
