use model::{MatchId, ReceivedLikeId};
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct MatchesIteratorState {
    pub id_at_reset: MatchId,
    pub page: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ReceivedLikesIteratorState {
    pub previous_id_at_reset: Option<ReceivedLikeId>,
    pub id_at_reset: ReceivedLikeId,
    pub page: i64,
}
