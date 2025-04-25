use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::ToSchema;

use crate::{AccountId, AccountIdDb, AccountIdInternal, ChatMessageReport, ContentId, ProfileAge, ReportIdDb, ReportProcessingState, ReportTypeNumber, ReportTypeNumberInternal};

#[derive(Debug, Clone)]
pub struct ReportInternal {
    pub info: ReportDetailedInfoInternal,
    pub id: ReportIdDb,
    pub creator_db_id: AccountIdDb,
    pub target_db_id: AccountIdDb,
}

impl ReportInternal {
    pub fn state(&self) -> ReportProcessingState {
        ReportProcessingState::Waiting
    }
}

#[derive(Debug, Clone)]
pub struct ReportDetailedInfoInternal {
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub report_type: ReportTypeNumberInternal,
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
pub enum ReportChatInfoInteractionState {
    None,
    CreatorLiked,
    TargetLiked,
    Match,
}

impl Default for ReportChatInfoInteractionState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct ReportChatInfo {
    pub state: ReportChatInfoInteractionState,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub creator_blocked_target: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub target_blocked_creator: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    #[schema(default = 0)]
    pub creator_sent_messages_count: i64,
    #[serde(default, skip_serializing_if = "is_zero")]
    #[schema(default = 0)]
    pub target_sent_messages_count: i64,
}

fn is_zero(value: &i64) -> bool {
    *value == 0
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
    /// Only available when chat component is enabled and account interaction
    /// exists.
    pub chat_info: Option<ReportChatInfo>
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub struct ReportContent {
    pub profile_name: Option<String>,
    pub profile_text: Option<String>,
    pub profile_content: Option<ContentId>,
    pub chat_message: Option<ChatMessageReport>,
    pub custom_report: Option<CustomReportContent>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub struct CustomReportContent {
    pub boolean_value: Option<bool>,
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
