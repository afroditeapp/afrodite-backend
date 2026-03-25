use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AccountId;

/// First byte of websocket binary protocol messages sent from client to server.
///
/// Remaining bytes are message payload. Payload format depends on the message
/// type value:
/// - `SyncVersionList` (0): payload contains list of current data sync versions
///   where items are `[u8; 2]`. The first `u8` is the data type number and the
///   second `u8` is the sync version number for that data. If client does not
///   have any version of the data, version number must be `255`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum ClientMessageType {
    // Reserved message type ranges (u8):
    // - common: 0..=29
    SyncVersionList = 0,
    // - account: 30..=59
    // - profile: 60..=89
    // - media: 90..=119
    // - chat: 120..=149
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub enum EventToServerType {
    /// Data: [EventToServer::a]
    TypingStart,
    TypingStop,
    /// Data: [EventToServer::a] and [EventToServer::o].
    CheckOnlineStatus,
}

/// Event from client to server sent via WebSocket
///
/// Uses single-character field names to minimize bandwidth
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EventToServer {
    t: EventToServerType,
    #[serde(skip_serializing_if = "Option::is_none")]
    a: Option<AccountId>,
    /// Online status (None value is false)
    #[serde(skip_serializing_if = "Option::is_none")]
    o: Option<bool>,
}

impl EventToServer {
    pub fn message_type(&self) -> EventToServerType {
        self.t
    }

    pub fn account(&self) -> Option<AccountId> {
        self.a
    }

    pub fn is_online(&self) -> bool {
        self.o.unwrap_or_default()
    }
}
