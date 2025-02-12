use model::{AccountId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileReportDetailed {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub profile_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetProfileReportList {
    pub values: Vec<ProfileReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessProfileReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub profile_text: Option<String>,
}
