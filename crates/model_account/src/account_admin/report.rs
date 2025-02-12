use model::{AccountId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::AccountReportContent;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AccountReportDetailed {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub content: AccountReportContent,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetAccountReportList {
    pub values: Vec<AccountReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessAccountReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub content: AccountReportContent,
}
