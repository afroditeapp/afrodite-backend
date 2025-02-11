use model::{AccountId, ContentId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MediaReportDetailed {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub profile_content: Vec<ContentId>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetMediaReportList {
    pub values: Vec<MediaReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessMediaReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub profile_content: Vec<ContentId>,
}
