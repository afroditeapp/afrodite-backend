use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AccountId;

use super::ReceivedLikesSyncVersion;


/// Session ID type for received likes iterator so that client can detect
/// server restarts and ask user to refresh received likes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReceivedLikesIteratorSessionIdInternal {
    id: uuid::Uuid,
}

impl ReceivedLikesIteratorSessionIdInternal {
    /// Current implementation uses UUID. Only requirement for this
    /// type is that next one should be different than the previous.
    pub fn create_random() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
        }
    }
}

/// Session ID type for received likes iterator so that client can detect
/// server restarts and ask user to refresh received likes.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ReceivedLikesIteratorSessionId {
    id: String,
}

impl From<ReceivedLikesIteratorSessionIdInternal> for ReceivedLikesIteratorSessionId {
    fn from(value: ReceivedLikesIteratorSessionIdInternal) -> Self {
        Self {
            id: value.id.hyphenated().to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct NewReceivedLikesAvailableResult {
    /// The sync version is for `new_received_likes_available`
    pub version: ReceivedLikesSyncVersion,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub new_received_likes_available: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResetReceivedLikesIteratorResult {
    /// The sync version is for `new_received_likes_available`
    pub version: ReceivedLikesSyncVersion,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub new_received_likes_available: bool,
    pub session_id: ReceivedLikesIteratorSessionId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ReceivedLikesPage {
    pub profiles: Vec<AccountId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_invalid_iterator_session_id: bool,
}
