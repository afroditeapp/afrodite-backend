use model::NewReceivedLikesCount;
use model_server_data::ReceivedLikesIteratorSessionId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AccountId;
use crate::ReceivedLikesSyncVersion;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResetReceivedLikesIteratorResult {
    pub v: ReceivedLikesSyncVersion,
    pub c: NewReceivedLikesCount,
    pub s: ReceivedLikesIteratorSessionId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ReceivedLikesPage {
    pub n: PageItemCountForNewLikes,
    pub p: Vec<AccountId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_invalid_iterator_session_id: bool,
}

/// Define how many returned profiles counted from the first page item are
/// new likes (interaction state changed to like after previous received likes
/// iterator reset).
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
)]
pub struct PageItemCountForNewLikes {
    pub c: i64,
}
