use num_enum::TryFromPrimitive;

use crate::{AccountId, CheckOnlineStatusResponse, EventToClientInternal, LastSeenTime};

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

/// First byte of websocket binary protocol messages sent from server to client.
///
/// Remaining bytes are message payload. Payload format depends on the message type
/// value:
/// - `PendingAppNotificationsChanged` (0): payload is empty.
/// - `ClientConfigChanged` (1): payload is empty.
/// - `NewsCountChanged` (2): payload is empty.
/// - `ScheduledMaintenanceStatus` (3): payload is JSON for
///   `ScheduledMaintenanceStatus`.
/// - `AdminBotNotification` (4): payload is JSON for
///   `AdminBotNotificationTypes`.
/// - `PushNotificationInfoChanged` (5): payload is empty.
/// - `AccountStateChanged` (30): payload is empty.
/// - `ProfileChanged` (60): payload is empty.
/// - `ContentProcessingStateChanged` (90): payload is JSON for
///   `ContentProcessingStateChanged`.
/// - `MediaContentChanged` (91): payload is empty.
/// - `NewMessageReceived` (120): payload is empty.
/// - `PendingChatNotificationsChanged` (121): payload is empty.
/// - `ReceivedLikesChanged` (122): payload is empty.
/// - `DailyLikesLeftChanged` (123): payload is empty.
/// - `TypingStart` (124): payload is exactly 16 bytes account UUID in
///   big-endian byte order.
/// - `TypingStop` (125): payload is exactly 16 bytes account UUID in
///   big-endian byte order.
/// - `CheckOnlineStatusResponse` (126): payload is 16 bytes account UUID,
///   followed by one byte which is 0 when last seen time is missing and 1 when
///   value is included. If included, payload ends with 8-byte big-endian i64.
/// - `MessageDeliveryInfoChanged` (127): payload is empty.
/// - `LatestSeenMessageChanged` (128): payload is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum ServerMessageType {
    // Reserved message type ranges (u8):
    // - common: 0..=29
    PendingAppNotificationsChanged = 0,
    ClientConfigChanged = 1,
    NewsCountChanged = 2,
    ScheduledMaintenanceStatus = 3,
    AdminBotNotification = 4,
    PushNotificationInfoChanged = 5,
    // - account: 30..=59
    /// Account state, profile visibility or permissions changed
    AccountStateChanged = 30,
    // - profile: 60..=89
    ProfileChanged = 60,
    // - media: 90..=119
    ContentProcessingStateChanged = 90,
    MediaContentChanged = 91,
    // - chat: 120..=149
    NewMessageReceived = 120,
    PendingChatNotificationsChanged = 121,
    ReceivedLikesChanged = 122,
    DailyLikesLeftChanged = 123,
    TypingStart = 124,
    TypingStop = 125,
    CheckOnlineStatusResponse = 126,
    MessageDeliveryInfoChanged = 127,
    LatestSeenMessageChanged = 128,
}

pub fn create_server_binary_message(
    event: &EventToClientInternal,
) -> Result<Vec<u8>, serde_json::Error> {
    let message_type = match event {
        EventToClientInternal::AccountStateChanged => ServerMessageType::AccountStateChanged,
        EventToClientInternal::ContentProcessingStateChanged(_) => {
            ServerMessageType::ContentProcessingStateChanged
        }
        EventToClientInternal::NewMessageReceived => ServerMessageType::NewMessageReceived,
        EventToClientInternal::PendingChatNotificationsChanged => {
            ServerMessageType::PendingChatNotificationsChanged
        }
        EventToClientInternal::PendingAppNotificationsChanged => {
            ServerMessageType::PendingAppNotificationsChanged
        }
        EventToClientInternal::ReceivedLikesChanged => ServerMessageType::ReceivedLikesChanged,
        EventToClientInternal::ClientConfigChanged => ServerMessageType::ClientConfigChanged,
        EventToClientInternal::ProfileChanged => ServerMessageType::ProfileChanged,
        EventToClientInternal::NewsChanged => ServerMessageType::NewsCountChanged,
        EventToClientInternal::MediaContentChanged => ServerMessageType::MediaContentChanged,
        EventToClientInternal::DailyLikesLeftChanged => ServerMessageType::DailyLikesLeftChanged,
        EventToClientInternal::ScheduledMaintenanceStatus(_) => {
            ServerMessageType::ScheduledMaintenanceStatus
        }
        EventToClientInternal::AdminBotNotification(_) => ServerMessageType::AdminBotNotification,
        EventToClientInternal::PushNotificationInfoChanged => {
            ServerMessageType::PushNotificationInfoChanged
        }
        EventToClientInternal::TypingStart(_) => ServerMessageType::TypingStart,
        EventToClientInternal::TypingStop(_) => ServerMessageType::TypingStop,
        EventToClientInternal::CheckOnlineStatusResponse(_) => {
            ServerMessageType::CheckOnlineStatusResponse
        }
        EventToClientInternal::MessageDeliveryInfoChanged => {
            ServerMessageType::MessageDeliveryInfoChanged
        }
        EventToClientInternal::LatestSeenMessageChanged => {
            ServerMessageType::LatestSeenMessageChanged
        }
    };

    let mut message = vec![message_type as u8];

    match event {
        EventToClientInternal::ContentProcessingStateChanged(value) => {
            message.extend(serde_json::to_vec(value)?);
        }
        EventToClientInternal::ScheduledMaintenanceStatus(value) => {
            message.extend(serde_json::to_vec(value)?);
        }
        EventToClientInternal::AdminBotNotification(value) => {
            message.extend(serde_json::to_vec(value)?);
        }
        EventToClientInternal::TypingStart(value) | EventToClientInternal::TypingStop(value) => {
            append_account_id_payload(&mut message, *value);
        }
        EventToClientInternal::CheckOnlineStatusResponse(value) => {
            append_check_online_status_response_payload(&mut message, value);
        }
        EventToClientInternal::AccountStateChanged
        | EventToClientInternal::NewMessageReceived
        | EventToClientInternal::PendingChatNotificationsChanged
        | EventToClientInternal::PendingAppNotificationsChanged
        | EventToClientInternal::ReceivedLikesChanged
        | EventToClientInternal::ClientConfigChanged
        | EventToClientInternal::ProfileChanged
        | EventToClientInternal::NewsChanged
        | EventToClientInternal::MediaContentChanged
        | EventToClientInternal::DailyLikesLeftChanged
        | EventToClientInternal::PushNotificationInfoChanged
        | EventToClientInternal::MessageDeliveryInfoChanged
        | EventToClientInternal::LatestSeenMessageChanged => (),
    }

    Ok(message)
}

pub fn parse_server_binary_message(
    message: &[u8],
) -> Result<EventToClientInternal, serde_json::Error> {
    let (message_type_u8, payload) = message.split_first().ok_or_else(|| {
        serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing server message type byte",
        ))
    })?;

    let message_type = ServerMessageType::try_from(*message_type_u8).map_err(|_| {
        serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("unsupported server message type {message_type_u8}"),
        ))
    })?;

    let event = match message_type {
        ServerMessageType::PendingAppNotificationsChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::PendingAppNotificationsChanged
        }
        ServerMessageType::ClientConfigChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::ClientConfigChanged
        }
        ServerMessageType::NewsCountChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::NewsChanged
        }
        ServerMessageType::ScheduledMaintenanceStatus => {
            EventToClientInternal::ScheduledMaintenanceStatus(serde_json::from_slice(payload)?)
        }
        ServerMessageType::AdminBotNotification => {
            EventToClientInternal::AdminBotNotification(serde_json::from_slice(payload)?)
        }
        ServerMessageType::PushNotificationInfoChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::PushNotificationInfoChanged
        }
        ServerMessageType::AccountStateChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::AccountStateChanged
        }
        ServerMessageType::ProfileChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::ProfileChanged
        }
        ServerMessageType::ContentProcessingStateChanged => {
            EventToClientInternal::ContentProcessingStateChanged(serde_json::from_slice(payload)?)
        }
        ServerMessageType::MediaContentChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::MediaContentChanged
        }
        ServerMessageType::NewMessageReceived => {
            expect_empty_payload(payload)?;
            EventToClientInternal::NewMessageReceived
        }
        ServerMessageType::PendingChatNotificationsChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::PendingChatNotificationsChanged
        }
        ServerMessageType::ReceivedLikesChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::ReceivedLikesChanged
        }
        ServerMessageType::DailyLikesLeftChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::DailyLikesLeftChanged
        }
        ServerMessageType::TypingStart => {
            EventToClientInternal::TypingStart(parse_account_id_payload(payload)?)
        }
        ServerMessageType::TypingStop => {
            EventToClientInternal::TypingStop(parse_account_id_payload(payload)?)
        }
        ServerMessageType::CheckOnlineStatusResponse => {
            EventToClientInternal::CheckOnlineStatusResponse(
                parse_check_online_status_response_payload(payload)?,
            )
        }
        ServerMessageType::MessageDeliveryInfoChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::MessageDeliveryInfoChanged
        }
        ServerMessageType::LatestSeenMessageChanged => {
            expect_empty_payload(payload)?;
            EventToClientInternal::LatestSeenMessageChanged
        }
    };

    Ok(event)
}

fn append_account_id_payload(buffer: &mut Vec<u8>, account_id: AccountId) {
    buffer.extend_from_slice(account_id.as_ref().as_bytes());
}

fn parse_account_id_payload(payload: &[u8]) -> Result<AccountId, serde_json::Error> {
    let bytes: [u8; 16] = payload.try_into().map_err(|_| {
        serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid account id payload size",
        ))
    })?;

    Ok(AccountId::new_base_64_url(
        simple_backend_utils::UuidBase64Url::from_bytes(bytes),
    ))
}

fn append_check_online_status_response_payload(
    buffer: &mut Vec<u8>,
    value: &CheckOnlineStatusResponse,
) {
    append_account_id_payload(buffer, value.a);
    match value.l {
        Some(last_seen) => {
            buffer.push(1);
            buffer.extend_from_slice(&last_seen.raw().to_be_bytes());
        }
        None => {
            buffer.push(0);
        }
    }
}

fn parse_check_online_status_response_payload(
    payload: &[u8],
) -> Result<CheckOnlineStatusResponse, serde_json::Error> {
    if payload.len() != 17 && payload.len() != 25 {
        return Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid check online status payload size",
        )));
    }

    let (account_id_payload, has_last_seen_and_tail) = payload.split_at(16);
    let account_id = parse_account_id_payload(account_id_payload)?;
    let has_last_seen = has_last_seen_and_tail.first().copied().unwrap_or_default() != 0;

    let last_seen = if has_last_seen {
        let last_seen_tail = has_last_seen_and_tail.get(1..).ok_or_else(|| {
            serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing last seen payload",
            ))
        })?;
        let raw_bytes: [u8; 8] = last_seen_tail.try_into().map_err(|_| {
            serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid last seen payload size",
            ))
        })?;
        Some(LastSeenTime::new(i64::from_be_bytes(raw_bytes)))
    } else {
        if has_last_seen_and_tail.len() != 1 {
            return Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "check online status payload contains unexpected trailing data",
            )));
        }
        None
    };

    Ok(CheckOnlineStatusResponse {
        a: account_id,
        l: last_seen,
    })
}

fn expect_empty_payload(payload: &[u8]) -> Result<(), serde_json::Error> {
    if payload.is_empty() {
        Ok(())
    } else {
        Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected empty payload",
        )))
    }
}
