use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{AccountId, ReportIdDb, ReportProcessingState, ReportTypeNumber};

#[derive(Debug, Clone)]
pub struct ReportInternal {
    pub info: ReportDetailedInfo,
    pub id: ReportIdDb,
}

impl ReportInternal {
    pub fn state(&self) -> ReportProcessingState {
        ReportProcessingState::Waiting
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ReportDetailedInfo {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub report_type: ReportTypeNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ReportDetailed {
    pub info: ReportDetailedInfo,
    pub content: ReportContent,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub struct ReportContent {
    pub profile_name: Option<String>,
    pub profile_text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetReportList {
    pub values: Vec<ReportDetailed>,
}
