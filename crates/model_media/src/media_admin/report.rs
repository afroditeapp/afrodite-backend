use model::{AccountId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::MediaReportContent;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MediaReportDetailed {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub content: MediaReportContent,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetMediaReportList {
    pub values: Vec<MediaReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessMediaReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub content: MediaReportContent,
}
