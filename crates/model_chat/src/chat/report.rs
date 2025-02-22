use model::AccountId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateChatMessageReport {
    pub target: AccountId,
    pub message: String,
}
