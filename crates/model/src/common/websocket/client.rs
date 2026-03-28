use num_enum::TryFromPrimitive;

/// First byte of websocket binary protocol messages sent from client to server.
///
/// Remaining bytes are message payload. Payload format depends on the message
/// type value:
/// - `SyncVersionList` (0): payload contains list of current data sync versions.
///   Each byte in the payload is a sync version for a data type. The position
///   of the byte defines the data type (see `SyncCheckDataType`). If client
///   does not have any version of the data, version number must be `255`.
/// - `ClearMaintenanceStatusIfPossible` (1): payload is empty.
/// - `TypingStart` (120): payload is exactly 16 bytes account UUID in big-endian
///   byte order.
/// - `TypingStop` (121): payload is empty.
/// - `CheckOnlineStatus` (122): payload is 16 bytes account UUID. Optional 17th
///   byte can be included for online status hint (0 = false, non-zero = true).
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum ClientMessageType {
    // Reserved message type ranges (u8):
    // - common: 0..=29
    SyncVersionList = 0,
    ClearMaintenanceStatusIfPossible = 1,
    // - account: 30..=59
    // - profile: 60..=89
    // - media: 90..=119
    // - chat: 120..=149
    TypingStart = 120,
    TypingStop = 121,
    CheckOnlineStatus = 122,
}

#[derive(Debug, Clone, Copy)]
pub enum ClientMessageForDataAllCrate<'a> {
    SyncVersionList(&'a [u8]),
}
