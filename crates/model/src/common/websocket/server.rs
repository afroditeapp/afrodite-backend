use num_enum::TryFromPrimitive;
use simple_backend_model::ScheduledMaintenanceStatus;
use utils::minimal_i64;
use utoipa::ToSchema;

use crate::{
    AccountId, ContentProcessingStateChanged, ContentProcessingStateInternal,
    EventToClientInternal, OnlineStatusUpdate, ProfileLink, ResponseNextProfilePageStatus,
    ResponseResetProfilePagingStatus,
};

mod parser;
pub use parser::parse_server_binary_message;

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
/// - `RequestAdminBotConfigWarnings` (6): payload format:
///   - request id byte (u8)
/// - `WebSocketConnectionAttemptsRemaining` (7): payload format:
///   - remaining daily websocket connection attempts as u8
/// - `AppUpdateAvailable` (8): payload is currently empty.
///   - Client must accept both empty and non-empty payload to support
///     forward-compatible protocol changes.
/// - `AccountStateChanged` (30): payload is empty.
/// - `AccountVerificationQueuePositionChanged` (31): payload format:
///   - optional queue position as 1 byte (empty payload means `None`)
/// - `ProfileChanged` (60): payload is empty.
/// - `ResponseResetProfilePaging` (61): payload format:
///   - request id byte (u8)
///   - status byte:
///     - 0: success
///     - 1: rate limited
///     - 2: internal server error
///   - if status is 0:
///     - profile iterator session id as minimal i64
/// - `ResponseNextProfilePage` (62): payload format:
///   - request id byte (u8)
///   - status byte:
///     - 0: success
///     - 1: invalid iterator session id
///     - 2: rate limited
///     - 3: internal server error
///   - if status is 0:
///     - repeated profile entries until payload ends:
///       - account id as 16-byte big-endian UUID
///       - profile version as 16-byte big-endian UUID
///       - profile content version as 16-byte big-endian UUID
///       - null last seen time (0 byte) or last seen time as minimal i64
/// - `ResponseAutomaticProfileSearchResetProfilePaging` (63): payload format:
///   - request id byte (u8)
///   - status byte:
///     - 0: success
///     - 1: rate limited
///     - 2: internal server error
///   - if status is 0:
///     - automatic profile search iterator session id as minimal i64
/// - `ResponseAutomaticProfileSearchNextProfilePage` (64): payload format:
///   - request id byte (u8)
///   - status byte:
///     - 0: success
///     - 1: invalid iterator session id
///     - 2: rate limited
///     - 3: internal server error
///   - if status is 0:
///     - repeated profile entries until payload ends:
///       - account id as 16-byte big-endian UUID
///       - profile version as 16-byte big-endian UUID
///       - profile content version as 16-byte big-endian UUID
///       - null last seen time (0 byte) or last seen time as minimal i64
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
/// - `OnlineStatusUpdated` (126): payload is 16 bytes account UUID,
///   followed by null last seen time (0 byte) or last seen time as minimal i64.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, ToSchema)]
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
    RequestAdminBotConfigWarnings = 6,
    WebSocketConnectionAttemptsRemaining = 7,
    AppUpdateAvailable = 8,
    // - account: 30..=59
    /// Account state, profile visibility or permissions changed
    AccountStateChanged = 30,
    AccountVerificationQueuePositionChanged = 31,
    // - profile: 60..=89
    ProfileChanged = 60,
    ResponseResetProfilePaging = 61,
    ResponseNextProfilePage = 62,
    ResponseAutomaticProfileSearchResetProfilePaging = 63,
    ResponseAutomaticProfileSearchNextProfilePage = 64,
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
    OnlineStatusUpdated = 126,
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
        EventToClientInternal::ResponseResetProfilePaging { .. } => {
            ServerMessageType::ResponseResetProfilePaging
        }
        EventToClientInternal::ResponseNextProfilePage { .. } => {
            ServerMessageType::ResponseNextProfilePage
        }
        EventToClientInternal::ResponseAutomaticProfileSearchResetProfilePaging { .. } => {
            ServerMessageType::ResponseAutomaticProfileSearchResetProfilePaging
        }
        EventToClientInternal::ResponseAutomaticProfileSearchNextProfilePage { .. } => {
            ServerMessageType::ResponseAutomaticProfileSearchNextProfilePage
        }
        EventToClientInternal::NewsChanged => ServerMessageType::NewsCountChanged,
        EventToClientInternal::MediaContentChanged => ServerMessageType::MediaContentChanged,
        EventToClientInternal::AccountVerificationQueuePositionChanged { .. } => {
            ServerMessageType::AccountVerificationQueuePositionChanged
        }
        EventToClientInternal::DailyLikesLeftChanged => ServerMessageType::DailyLikesLeftChanged,
        EventToClientInternal::ScheduledMaintenanceStatus(_) => {
            ServerMessageType::ScheduledMaintenanceStatus
        }
        EventToClientInternal::AdminBotNotification(_) => ServerMessageType::AdminBotNotification,
        EventToClientInternal::RequestAdminBotConfigWarnings { .. } => {
            ServerMessageType::RequestAdminBotConfigWarnings
        }
        EventToClientInternal::WebSocketConnectionAttemptsRemaining { .. } => {
            ServerMessageType::WebSocketConnectionAttemptsRemaining
        }
        EventToClientInternal::AppUpdateAvailable => ServerMessageType::AppUpdateAvailable,
        EventToClientInternal::PushNotificationInfoChanged => {
            ServerMessageType::PushNotificationInfoChanged
        }
        EventToClientInternal::TypingStart(_) => ServerMessageType::TypingStart,
        EventToClientInternal::TypingStop(_) => ServerMessageType::TypingStop,
        EventToClientInternal::OnlineStatusUpdated(_) => ServerMessageType::OnlineStatusUpdated,
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
        EventToClientInternal::AccountVerificationQueuePositionChanged { queue_position } => {
            if let Some(queue_position) = queue_position {
                message.push(*queue_position);
            }
        }
        EventToClientInternal::ScheduledMaintenanceStatus(value) => {
            append_scheduled_maintenance_status_payload(&mut message, value);
        }
        EventToClientInternal::AdminBotNotification(value) => {
            message.push(value.bits());
        }
        EventToClientInternal::RequestAdminBotConfigWarnings { request_id } => {
            message.push(*request_id);
        }
        EventToClientInternal::WebSocketConnectionAttemptsRemaining { remaining } => {
            message.push(*remaining);
        }
        EventToClientInternal::TypingStart(value) | EventToClientInternal::TypingStop(value) => {
            append_account_id_payload(&mut message, *value);
        }
        EventToClientInternal::OnlineStatusUpdated(value) => {
            append_online_status_updated_payload(&mut message, value);
        }
        EventToClientInternal::ResponseResetProfilePaging {
            request_id,
            status,
            iterator_session_id,
        } => {
            append_response_reset_profile_paging_payload(
                &mut message,
                *request_id,
                *status,
                *iterator_session_id,
            );
        }
        EventToClientInternal::ResponseNextProfilePage {
            request_id,
            status,
            profiles,
        } => {
            append_response_next_profile_page_payload(&mut message, *request_id, *status, profiles);
        }
        EventToClientInternal::ResponseAutomaticProfileSearchResetProfilePaging {
            request_id,
            status,
            iterator_session_id,
        } => {
            append_response_reset_profile_paging_payload(
                &mut message,
                *request_id,
                *status,
                *iterator_session_id,
            );
        }
        EventToClientInternal::ResponseAutomaticProfileSearchNextProfilePage {
            request_id,
            status,
            profiles,
        } => {
            append_response_next_profile_page_payload(&mut message, *request_id, *status, profiles);
        }
        EventToClientInternal::AccountStateChanged
        | EventToClientInternal::NewMessageReceived
        | EventToClientInternal::PendingChatNotificationsChanged
        | EventToClientInternal::PendingAppNotificationsChanged
        | EventToClientInternal::AppUpdateAvailable
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

fn append_account_id_payload(buffer: &mut Vec<u8>, account_id: AccountId) {
    buffer.extend_from_slice(account_id.as_ref().as_bytes());
}

fn append_response_reset_profile_paging_payload(
    buffer: &mut Vec<u8>,
    request_id: u8,
    status: ResponseResetProfilePagingStatus,
    iterator_session_id: Option<i64>,
) {
    buffer.push(request_id);
    buffer.push(status as u8);

    if !matches!(status, ResponseResetProfilePagingStatus::Success) {
        return;
    }

    if let Some(iterator_session_id) = iterator_session_id {
        minimal_i64::add_minimal_i64(buffer, iterator_session_id);
    }
}

fn append_response_next_profile_page_payload(
    buffer: &mut Vec<u8>,
    request_id: u8,
    status: ResponseNextProfilePageStatus,
    profiles: &[ProfileLink],
) {
    buffer.push(request_id);
    buffer.push(status as u8);

    if !matches!(status, ResponseNextProfilePageStatus::Success) {
        return;
    }

    for profile in profiles {
        let account_id = profile.account_id();
        let profile_version = profile.profile_version();
        let profile_content_version = profile.profile_content_version();

        buffer.extend_from_slice(account_id.aid.as_bytes());
        buffer.extend_from_slice(profile_version.as_ref().as_bytes());
        buffer.extend_from_slice(profile_content_version.as_ref().as_bytes());

        if let Some(last_seen) = profile.last_seen_time() {
            minimal_i64::add_minimal_i64(buffer, last_seen.raw());
        } else {
            buffer.push(0);
        }
    }
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
        ContentProcessingStateInternal::Completed {
            content_id,
            face_detected,
        } => {
            buffer.extend_from_slice(content_id.cid.as_bytes());
            buffer.push(u8::from(face_detected));
        }
        ContentProcessingStateInternal::Empty
        | ContentProcessingStateInternal::Processing
        | ContentProcessingStateInternal::Failed
        | ContentProcessingStateInternal::NsfwDetected => (),
    }
}

fn append_online_status_updated_payload(buffer: &mut Vec<u8>, value: &OnlineStatusUpdate) {
    append_account_id_payload(buffer, value.a);
    match value.l {
        Some(last_seen) => {
            minimal_i64::add_minimal_i64(buffer, last_seen.raw());
        }
        None => {
            buffer.push(0);
        }
    }
}
