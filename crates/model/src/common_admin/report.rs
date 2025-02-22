use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::ToSchema;

use crate::{AccountId, AccountIdDb, AccountIdInternal, ContentId, ProfileAge, ReportIdDb, ReportProcessingState, ReportTypeNumber};

#[derive(Debug, Clone)]
pub struct ReportInternal {
    pub info: ReportDetailedInfo,
    pub id: ReportIdDb,
    pub creator_db_id: AccountIdDb,
    pub target_db_id: AccountIdDb,
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
pub struct ReportAccountInfo {
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    pub name: String,
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
    /// Only available when profile component is enabled.
    pub creator_info: Option<ReportAccountInfo>,
    /// Only available when profile component is enabled.
    pub target_info: Option<ReportAccountInfo>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub struct ReportContent {
    pub profile_name: Option<String>,
    pub profile_text: Option<String>,
    pub profile_content: Option<ContentId>,
    pub chat_message: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct GetReportList {
    pub values: Vec<ReportDetailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProcessReport {
    pub creator: AccountId,
    pub target: AccountId,
    pub report_type: ReportTypeNumber,
    pub content: ReportContent,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum ReportIteratorMode {
    Received,
    Sent,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
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
