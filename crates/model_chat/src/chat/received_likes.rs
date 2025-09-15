use model::ReceivedLikeId;
use model_server_data::{ProfileLink, ReceivedLikesIteratorState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ResetReceivedLikesIteratorResult {
    pub s: ReceivedLikesIteratorState,
}

#[derive(Serialize, ToSchema)]
pub struct ReceivedLikesPage {
    pub l: Vec<ReceivedLikesPageItem>,
}

#[derive(Serialize, ToSchema)]
pub struct ReceivedLikesPageItem {
    pub p: ProfileLink,
    /// If Some, the like is not viewed yet
    pub not_viewed: Option<ReceivedLikeId>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct MarkReceivedLikesViewed {
    pub v: Vec<ReceivedLikeId>,
}
