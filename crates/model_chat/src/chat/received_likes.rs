use model::NewReceivedLikesCount;
use model_server_data::{ProfileLink, ReceivedLikesIteratorState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ReceivedLikesSyncVersion;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResetReceivedLikesIteratorResult {
    pub v: ReceivedLikesSyncVersion,
    pub c: NewReceivedLikesCount,
    pub s: ReceivedLikesIteratorState,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ReceivedLikesPage {
    pub n: PageItemCountForNewLikes,
    pub p: Vec<ProfileLink>,
}

/// Define how many returned profiles counted from the first page item are
/// new likes (interaction state changed to like after previous received likes
/// iterator reset).
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct PageItemCountForNewLikes {
    pub c: i64,
}
