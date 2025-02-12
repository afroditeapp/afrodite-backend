use model::{AccountId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::ProfileReportContent;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileReportDetailed {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub content: ProfileReportContent,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetProfileReportList {
    pub values: Vec<ProfileReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessProfileReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub content: ProfileReportContent,
}
