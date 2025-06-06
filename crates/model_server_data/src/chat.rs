use model::NextNumberStorage;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum LimitedActionStatus {
    /// Action completed successfully.
    Success,
    /// Action completed successfully but the action limit was reached.
    SuccessAndLimitReached,
    /// Action failed because the action limit is already reached.
    FailureLimitAlreadyReached,
}

/// Session ID type for matches iterator so that client can detect
/// server restarts and ask user to refresh matches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchesIteratorSessionIdInternal {
    id: i64,
}

impl MatchesIteratorSessionIdInternal {
    /// Current implementation uses i64. Only requirement for this
    /// type is that next one should be different than the previous.
    pub fn create(storage: &mut NextNumberStorage) -> Self {
        Self {
            id: storage.get_and_increment(),
        }
    }
}

/// Session ID type for matches iterator so that client can detect
/// server restarts and ask user to matches.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct MatchesIteratorSessionId {
    id: i64,
}

impl From<MatchesIteratorSessionIdInternal> for MatchesIteratorSessionId {
    fn from(value: MatchesIteratorSessionIdInternal) -> Self {
        Self { id: value.id }
    }
}

impl From<MatchesIteratorSessionId> for MatchesIteratorSessionIdInternal {
    fn from(value: MatchesIteratorSessionId) -> Self {
        Self { id: value.id }
    }
}

/// Session ID type for received likes iterator so that client can detect
/// server restarts and ask user to refresh received likes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReceivedLikesIteratorSessionIdInternal {
    id: i64,
}

impl ReceivedLikesIteratorSessionIdInternal {
    /// Current implementation uses i64. Only requirement for this
    /// type is that next one should be different than the previous.
    pub fn create(storage: &mut NextNumberStorage) -> Self {
        Self {
            id: storage.get_and_increment(),
        }
    }
}

/// Session ID type for received likes iterator so that client can detect
/// server restarts and ask user to refresh received likes.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ReceivedLikesIteratorSessionId {
    id: i64,
}

impl From<ReceivedLikesIteratorSessionIdInternal> for ReceivedLikesIteratorSessionId {
    fn from(value: ReceivedLikesIteratorSessionIdInternal) -> Self {
        Self { id: value.id }
    }
}

impl From<ReceivedLikesIteratorSessionId> for ReceivedLikesIteratorSessionIdInternal {
    fn from(value: ReceivedLikesIteratorSessionId) -> Self {
        Self { id: value.id }
    }
}
