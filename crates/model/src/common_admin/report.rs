use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{AccountId, ReportIdDb, ReportProcessingState};

#[derive(Debug, Clone)]
pub struct WaitingReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub id: ReportIdDb,
}

impl WaitingReport {
    pub fn state(&self) -> ReportProcessingState {
        ReportProcessingState::Waiting
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ReportDetailedInfo {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
}
