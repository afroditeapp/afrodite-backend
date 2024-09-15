use serde::{Deserialize, Serialize};
use utoipa::ToSchema;


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
