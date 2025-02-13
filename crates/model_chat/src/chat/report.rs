use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use model::{AccountId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct ChatReport {
    pub processing_state: ReportProcessingState,
    pub content: ChatReportContent,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateChatReport {
    pub target: AccountId,
    pub content: ChatReportContent,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, ToSchema, Insertable, AsChangeset, Selectable, Queryable)]
#[diesel(table_name = crate::schema::chat_report)]
#[diesel(check_for_backend(crate::Db))]
pub struct ChatReportContent {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub is_against_video_calling: bool,
}

impl ChatReportContent {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateChatReportResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_not_match: bool,
}

impl UpdateChatReportResult {
    pub fn success() -> Self {
        Self {
            error_not_match: false,
        }
    }

    pub fn not_match() -> Self {
        Self {
            error_not_match: true
        }
    }
}
