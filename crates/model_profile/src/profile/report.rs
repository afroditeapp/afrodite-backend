use model::{AccountId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct ProfileReport {
    pub processing_state: ReportProcessingState,
    pub content: ProfileReportContent,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateProfileReport {
    pub target: AccountId,
    pub content: ProfileReportContent,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct ProfileReportContent {
    pub profile_text: Option<String>,
}
