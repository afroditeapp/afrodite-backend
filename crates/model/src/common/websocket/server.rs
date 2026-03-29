use num_enum::TryFromPrimitive;
use utils::minimal_i64;

use crate::{
    AccountId, CheckOnlineStatusResponse, ContentProcessingStateChanged,
    ContentProcessingStateInternal, ContentProcessingStateType, EventToClientInternal,
    LastSeenTime, ScheduledMaintenanceStatus, UnixTime,
};

/// First byte of websocket binary protocol messages sent from server to client.
///
/// # Message types and payloads
///
/// - `PendingAppNotificationsChanged` (0): payload is empty.
/// - `ClientConfigChanged` (1): payload is empty.
/// - `NewsCountChanged` (2): payload is empty.
/// - `ScheduledMaintenanceStatus` (3): payload format:
///   - admin bot offline (u8, 0 or 1)
///   - maintenance start as optional minimal i64
///   - if start exists, maintenance end as optional minimal i64
/// - `AdminBotNotification` (4): payload is unsigned integer with
///   little-endian byte order for `AdminBotNotificationTypes` bitflags.
///   (1 byte = u8, 2 bytes = u16 etc.)
/// - `PushNotificationInfoChanged` (5): payload is empty.
/// - `AccountStateChanged` (30): payload is empty.
/// - `ProfileChanged` (60): payload is empty.
/// - `ContentProcessingStateChanged` (90): payload format:
///   - content processing server process ID as minimal i64
///   - content processing state byte:
///     - 0: Empty
///     - 1: InQueue
///     - 2: Processing
///     - 3: Completed
///     - 4: Failed
///     - 5: NsfwDetected
///   - state specific data:
///     - InQueue: queue number as minimal i64
///     - Completed:
///       - content ID as 16 byte big-endian UUID (16 bytes)
///       - face detection bool (1 byte, 0 or 1)
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
///
/// # Data formats
///
/// Data types used in payload definitions:
/// - minimal i64:
///   - i64 byte count (u8, values: 1, 2, 4, 8)
///   - i64 bytes (little-endian byte order)
/// - optional values in payloads are omitted when they are not present
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

pub fn create_server_binary_message(event: &EventToClientInternal) -> Vec<u8> {
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
            append_content_processing_state_changed_payload(&mut message, value);
        }
        EventToClientInternal::ScheduledMaintenanceStatus(value) => {
            append_scheduled_maintenance_status_payload(&mut message, value);
        }
        EventToClientInternal::AdminBotNotification(value) => {
            message.push(value.bits());
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

    message
}

pub fn parse_server_binary_message(message: &[u8]) -> Result<EventToClientInternal, String> {
    let (message_type_u8, payload) = message
        .split_first()
        .ok_or_else(|| "missing server message type byte".to_owned())?;

    let message_type = ServerMessageType::try_from(*message_type_u8)
        .map_err(|_| format!("unsupported server message type {message_type_u8}"))?;

    let event = match message_type {
        ServerMessageType::PendingAppNotificationsChanged => {
            EventToClientInternal::PendingAppNotificationsChanged
        }
        ServerMessageType::ClientConfigChanged => EventToClientInternal::ClientConfigChanged,
        ServerMessageType::NewsCountChanged => EventToClientInternal::NewsChanged,
        ServerMessageType::ScheduledMaintenanceStatus => {
            EventToClientInternal::ScheduledMaintenanceStatus(
                parse_scheduled_maintenance_status_payload(payload)?,
            )
        }
        ServerMessageType::AdminBotNotification => {
            let bits = payload
                .first()
                .copied()
                .ok_or_else(|| "missing admin bot notification payload".to_owned())?;
            EventToClientInternal::AdminBotNotification(
                crate::AdminBotNotificationTypes::from_bits_truncate(bits),
            )
        }
        ServerMessageType::PushNotificationInfoChanged => {
            EventToClientInternal::PushNotificationInfoChanged
        }
        ServerMessageType::AccountStateChanged => EventToClientInternal::AccountStateChanged,
        ServerMessageType::ProfileChanged => EventToClientInternal::ProfileChanged,
        ServerMessageType::ContentProcessingStateChanged => {
            EventToClientInternal::ContentProcessingStateChanged(
                parse_content_processing_state_changed_payload(payload)?,
            )
        }
        ServerMessageType::MediaContentChanged => EventToClientInternal::MediaContentChanged,
        ServerMessageType::NewMessageReceived => EventToClientInternal::NewMessageReceived,
        ServerMessageType::PendingChatNotificationsChanged => {
            EventToClientInternal::PendingChatNotificationsChanged
        }
        ServerMessageType::ReceivedLikesChanged => EventToClientInternal::ReceivedLikesChanged,
        ServerMessageType::DailyLikesLeftChanged => EventToClientInternal::DailyLikesLeftChanged,
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
            EventToClientInternal::MessageDeliveryInfoChanged
        }
        ServerMessageType::LatestSeenMessageChanged => {
            EventToClientInternal::LatestSeenMessageChanged
        }
    };

    Ok(event)
}

fn append_account_id_payload(buffer: &mut Vec<u8>, account_id: AccountId) {
    buffer.extend_from_slice(account_id.as_ref().as_bytes());
}

fn append_scheduled_maintenance_status_payload(
    buffer: &mut Vec<u8>,
    value: &ScheduledMaintenanceStatus,
) {
    buffer.push(u8::from(value.admin_bot_offline()));

    if let Some(start) = value.start().map(|time| time.ut) {
        minimal_i64::add_minimal_i64(buffer, start);
        if let Some(end) = value.end().map(|time| time.ut) {
            minimal_i64::add_minimal_i64(buffer, end);
        }
    }
}

fn parse_account_id_payload(payload: &[u8]) -> Result<AccountId, String> {
    let bytes: [u8; 16] = payload
        .try_into()
        .map_err(|_| "invalid account id payload size".to_owned())?;

    Ok(AccountId::new_base_64_url(
        simple_backend_utils::UuidBase64Url::from_bytes(bytes),
    ))
}

fn parse_scheduled_maintenance_status_payload(
    payload: &[u8],
) -> Result<ScheduledMaintenanceStatus, String> {
    let (admin_bot_offline_raw, remaining_payload) = payload
        .split_first()
        .ok_or_else(|| "missing scheduled maintenance admin_bot_offline payload".to_owned())?;

    let mut payload_iter = remaining_payload.iter().copied();
    let start = if remaining_payload.is_empty() {
        None
    } else {
        Some(parse_minimal_i64_value(&mut payload_iter)?)
    };

    let end = if payload_iter.clone().next().is_some() {
        Some(parse_minimal_i64_value(&mut payload_iter)?)
    } else {
        None
    };

    let mut status = ScheduledMaintenanceStatus::default();
    status.set_admin_bot_offline(*admin_bot_offline_raw != 0);
    status.set_maintenance_time(start.map(UnixTime::new), end.map(UnixTime::new));

    Ok(status)
}

fn parse_minimal_i64_value(payload_iter: &mut impl Iterator<Item = u8>) -> Result<i64, String> {
    minimal_i64::parse_minimal_i64_from_iter(payload_iter)
        .ok_or_else(|| "invalid or truncated minimal i64 payload".to_owned())
}

fn append_content_processing_state_changed_payload(
    buffer: &mut Vec<u8>,
    value: &ContentProcessingStateChanged,
) {
    minimal_i64::add_minimal_i64(buffer, value.id);
    let state_type = value.new_state.state_type();
    buffer.push(state_type as u8);

    match value.new_state {
        ContentProcessingStateInternal::InQueue {
            wait_queue_position,
        } => {
            minimal_i64::add_minimal_i64(buffer, wait_queue_position);
        }
        ContentProcessingStateInternal::Completed { content_id, fd } => {
            buffer.extend_from_slice(content_id.cid.as_bytes());
            buffer.push(u8::from(fd));
        }
        ContentProcessingStateInternal::Empty
        | ContentProcessingStateInternal::Processing
        | ContentProcessingStateInternal::Failed
        | ContentProcessingStateInternal::NsfwDetected => (),
    }
}

fn parse_content_processing_state_changed_payload(
    payload: &[u8],
) -> Result<ContentProcessingStateChanged, String> {
    let mut payload_iter = payload.iter().copied();

    let id = parse_minimal_i64_value(&mut payload_iter)?;
    let state_raw = payload_iter
        .next()
        .ok_or_else(|| "missing content processing state payload".to_owned())?;
    let state = ContentProcessingStateType::try_from(state_raw)
        .map_err(|_| format!("unsupported content processing state value {state_raw}"))?;

    let new_state = match state {
        ContentProcessingStateType::InQueue => ContentProcessingStateInternal::InQueue {
            wait_queue_position: parse_minimal_i64_value(&mut payload_iter)?,
        },
        ContentProcessingStateType::Completed => {
            let mut cid_bytes = [0u8; 16];
            for byte in cid_bytes.iter_mut() {
                *byte = payload_iter
                    .next()
                    .ok_or_else(|| "missing content id payload".to_owned())?;
            }
            let fd_byte = payload_iter
                .next()
                .ok_or_else(|| "missing face detection payload".to_owned())?;
            ContentProcessingStateInternal::Completed {
                content_id: crate::ContentId {
                    cid: simple_backend_utils::UuidBase64Url::from_bytes(cid_bytes),
                },
                fd: fd_byte != 0,
            }
        }
        ContentProcessingStateType::Empty => ContentProcessingStateInternal::Empty,
        ContentProcessingStateType::Processing => ContentProcessingStateInternal::Processing,
        ContentProcessingStateType::Failed => ContentProcessingStateInternal::Failed,
        ContentProcessingStateType::NsfwDetected => ContentProcessingStateInternal::NsfwDetected,
    };

    Ok(ContentProcessingStateChanged { id, new_state })
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
) -> Result<CheckOnlineStatusResponse, String> {
    if payload.len() != 17 && payload.len() != 25 {
        return Err("invalid check online status payload size".to_owned());
    }

    let (account_id_payload, has_last_seen_and_tail) = payload.split_at(16);
    let account_id = parse_account_id_payload(account_id_payload)?;
    let has_last_seen = has_last_seen_and_tail.first().copied().unwrap_or_default() != 0;

    let last_seen = if has_last_seen {
        let last_seen_tail = has_last_seen_and_tail
            .get(1..)
            .ok_or_else(|| "missing last seen payload".to_owned())?;
        let raw_bytes: [u8; 8] = last_seen_tail
            .try_into()
            .map_err(|_| "invalid last seen payload size".to_owned())?;
        Some(LastSeenTime::new(i64::from_be_bytes(raw_bytes)))
    } else {
        None
    };

    Ok(CheckOnlineStatusResponse {
        a: account_id,
        l: last_seen,
    })
}
