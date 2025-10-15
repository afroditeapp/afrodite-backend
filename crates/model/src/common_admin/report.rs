use serde::{Deserialize, Serialize};
use simple_backend_model::{NonEmptyString, UnixTime};
use utoipa::ToSchema;

use crate::{
    AccountId, AccountIdDb, AccountIdInternal, ChatMessageReport, ContentId, ProfileAge, ReportId,
    ReportIdDb, ReportProcessingState, ReportTypeNumber, ReportTypeNumberInternal,
};

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
    pub creation_time: UnixTime,
}

#[derive(Serialize, ToSchema)]
pub struct ReportDetailedInfo {
    pub id: ReportId,
    pub creator: AccountId,
    pub target: AccountId,
    pub processing_state: ReportProcessingState,
    pub report_type: ReportTypeNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ReportAccountInfo {
    #[schema(value_type = i64)]
    pub age: ProfileAge,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<NonEmptyString>,
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

pub struct ReportDetailedWithId {
    pub id: ReportIdDb,
    pub report: ReportDetailed,
}

#[derive(Serialize, ToSchema)]
pub struct ReportDetailed {
    pub info: ReportDetailedInfo,
    pub content: ReportContent,
    pub creator_info: ReportAccountInfo,
    pub target_info: ReportAccountInfo,
    /// Only available when account interaction exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_info: Option<ReportChatInfo>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub struct ReportContent {
    /// Null or non-empty string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_name: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_content: Option<ContentId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_message: Option<ChatMessageReport>,
}

#[derive(Serialize, ToSchema)]
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
