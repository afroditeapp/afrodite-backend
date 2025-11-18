use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AccountId;

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
