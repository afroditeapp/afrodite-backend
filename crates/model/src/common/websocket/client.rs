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
/// - `ResponseAdminBotConfigWarnings` (2): payload format:
///   - request id byte (u8)
///   - warnings flags as u16 (or larger) little-endian. Bits in the flags:
///     - bit 0: profile name moderation file config missing
///     - bit 1: profile text moderation file config missing
///     - bit 2: content moderation file config missing
///     - bit 3: face verification file config missing
///     - bit 4: account verification file config missing
///     - bit 5: account verification security content file config missing
///     - bit 6: report processing file config missing
///     - bit 7: report processing profile name file config missing
///     - bit 8: report processing profile text file config missing
///     - bit 9: report processing profile content file config missing
///     - bit 10: report processing messages file config missing
/// - `RequestResetProfilePaging` (60): payload format:
///   - request id byte (u8)
/// - `RequestGetNextProfilePage` (61): payload format:
///   - request id byte (u8)
///   - profile iterator session id as minimal i64
/// - `RequestAutomaticProfileSearchResetProfilePaging` (62): payload format:
///   - request id byte (u8)
/// - `RequestAutomaticProfileSearchGetNextProfilePage` (63): payload format:
///   - request id byte (u8)
///   - automatic profile search iterator session id as minimal i64
/// - `TypingStart` (120): payload is exactly 16 bytes account UUID in big-endian
///   byte order.
/// - `TypingStop` (121): payload is empty.
/// - `CheckOnlineStatus` (122): payload is 16 bytes account UUID. Optional
///   17th byte can be included for online status hint (0 = false, non-zero = true).
///
/// # Data formats
///
/// Data types used in payload definitions:
/// - minimal i64:
///   - i64 byte count (u8, values: 1, 2, 3, 4, 5, 6, 7, 8)
///   - i64 bytes (little-endian byte order)
/// - optional values in payloads are omitted when they are not present
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, ToSchema)]
#[repr(u8)]
pub enum ClientMessageType {
    // Reserved message type ranges (u8):
    // - common: 0..=29
    SyncVersionList = 0,
    ClearMaintenanceStatusIfPossible = 1,
    ResponseAdminBotConfigWarnings = 2,
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
    CheckOnlineStatus = 122,
}

#[derive(Debug, Clone, Copy)]
pub enum ClientMessageForDataAllCrate<'a> {
    SyncVersionList(&'a [u8]),
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct AdminBotConfigWarningFlags: u16 {
        const PROFILE_NAME_MODERATION_FILE_CONFIG_MISSING = 0x0001;
        const PROFILE_TEXT_MODERATION_FILE_CONFIG_MISSING = 0x0002;
        const CONTENT_MODERATION_FILE_CONFIG_MISSING = 0x0004;
        const FACE_VERIFICATION_FILE_CONFIG_MISSING = 0x0008;
        const ACCOUNT_VERIFICATION_FILE_CONFIG_MISSING = 0x0010;
        const ACCOUNT_VERIFICATION_SECURITY_CONTENT_FILE_CONFIG_MISSING = 0x0020;
        const REPORT_PROCESSING_FILE_CONFIG_MISSING = 0x0040;
        const REPORT_PROCESSING_PROFILE_NAME_FILE_CONFIG_MISSING = 0x0080;
        const REPORT_PROCESSING_PROFILE_TEXT_FILE_CONFIG_MISSING = 0x0100;
        const REPORT_PROCESSING_PROFILE_CONTENT_FILE_CONFIG_MISSING = 0x0200;
        const REPORT_PROCESSING_MESSAGES_FILE_CONFIG_MISSING = 0x0400;
    }
}
