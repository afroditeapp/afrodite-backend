







#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
)]
#[repr(u8)]
pub enum ClientTypeNumber {
    Android = 0,
    Ios = 1,
    /// Type number for test mode bots. It is the last available value
    /// to keep it more hidden.
    TestModeBot = 255,
}

impl TryFrom<u8> for ClientTypeNumber {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Android),
            1 => Ok(Self::Ios),
            255 => Ok(Self::TestModeBot),
            _ => Err(format!("Unknown client type number {}", value)),
        }
    }
}



/// Parsed info from first WebSocket Binary message from client without
/// the protocol version byte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebSocketClientInfo {
    pub client_type: ClientTypeNumber,
    pub major_version: u16,
    pub minor_version: u16,
    pub patch_version: u16,
}

impl WebSocketClientInfo {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        match bytes {
            [client_type0, major0, major1, minor0, minor1, patch0, patch1] => {
                let client_type = ClientTypeNumber::try_from(*client_type0)?;
                let major_version = u16::from_le_bytes([*major0, *major1]);
                let minor_version = u16::from_le_bytes([*minor0, *minor1]);
                let patch_version = u16::from_le_bytes([*patch0, *patch1]);
                Ok(Self { client_type, major_version, minor_version, patch_version })
            }
            _ => Err(format!("Invalid input byte count {}", bytes.len())),
        }
    }
}
