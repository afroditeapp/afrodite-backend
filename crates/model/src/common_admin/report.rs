use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdInternal, ReportIdDb, ReportProcessingState, ReportTypeNumber};

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
pub struct ReportDetailedWithId {
    pub id: ReportIdDb,
    pub report: ReportDetailed,
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum ReportIteratorMode {
    Received,
    Sent,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ReportIteratorQuery {
    pub start_position: UnixTime,
    pub page: i64,
    pub aid: AccountId,
    pub mode: ReportIteratorMode,
}

#[derive(Debug, Clone)]
pub struct ReportIteratorQueryInternal {
    pub start_position: UnixTime,
    pub page: i64,
    pub aid: AccountIdInternal,
    pub mode: ReportIteratorMode,
}
