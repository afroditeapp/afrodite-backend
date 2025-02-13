use model::{AccountId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ChatReportContent;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ChatReportDetailed {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub content: ChatReportContent,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetChatReportList {
    pub values: Vec<ChatReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessChatReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub content: ChatReportContent,
}
