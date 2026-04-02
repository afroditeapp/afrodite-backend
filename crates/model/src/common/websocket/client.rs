use num_enum::TryFromPrimitive;
use utoipa::ToSchema;

/// First byte of websocket binary protocol messages sent from client to server.
///
/// Remaining bytes are message payload. Payload format depends on the message
/// type value:
/// - `SyncVersionList` (0): payload contains list of current data sync versions.
///   Each byte in the payload is a sync version for a data type. The position
///   of the byte defines the data type (see `SyncCheckDataType`). If client
///   does not have any version of the data, version number must be `255`.
/// - `ClearMaintenanceStatusIfPossible` (1): payload is empty.
/// - `RequestResetProfilePaging` (60): payload is empty.
/// - `RequestGetNextProfilePage` (61): payload is profile iterator session id as
///   minimal i64.
/// - `RequestAutomaticProfileSearchResetProfilePaging` (62): payload is empty.
/// - `RequestAutomaticProfileSearchGetNextProfilePage` (63): payload is
///   automatic profile search iterator session id as minimal i64.
/// - `TypingStart` (120): payload is exactly 16 bytes account UUID in big-endian
///   byte order.
/// - `TypingStop` (121): payload is empty.
/// - `RequestCheckOnlineStatus` (122): payload is 16 bytes account UUID. Optional
///   17th byte can be included for online status hint (0 = false, non-zero = true).
///
/// # Data formats
///
/// Data types used in payload definitions:
/// - minimal i64:
///   - i64 byte count (u8, values: 1, 2, 4, 8)
///   - i64 bytes (little-endian byte order)
/// - optional values in payloads are omitted when they are not present
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, ToSchema)]
#[repr(u8)]
pub enum ClientMessageType {
    // Reserved message type ranges (u8):
    // - common: 0..=29
    SyncVersionList = 0,
    ClearMaintenanceStatusIfPossible = 1,
    // - account: 30..=59
    // - profile: 60..=89
    RequestResetProfilePaging = 60,
    RequestGetNextProfilePage = 61,
    RequestAutomaticProfileSearchResetProfilePaging = 62,
    RequestAutomaticProfileSearchGetNextProfilePage = 63,
    // - media: 90..=119
    // - chat: 120..=149
    TypingStart = 120,
    TypingStop = 121,
    RequestCheckOnlineStatus = 122,
}

#[derive(Debug, Clone, Copy)]
pub enum ClientMessageForDataAllCrate<'a> {
    SyncVersionList(&'a [u8]),
}
