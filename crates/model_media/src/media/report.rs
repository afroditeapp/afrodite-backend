use model::{AccountId, ContentId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateProfileContentReport {
    pub target: AccountId,
    pub content: ContentId,
}
