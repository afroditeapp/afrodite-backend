use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use model::{AccountId, CustomReportsConfig, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct AccountReport {
    pub processing_state: ReportProcessingState,
    pub content: AccountReportContent,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateAccountReport {
    pub target: AccountId,
    pub content: AccountReportContent,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize, ToSchema, Insertable, AsChangeset, Selectable, Queryable)]
#[diesel(table_name = crate::schema::account_report)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountReportContent {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub is_bot: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub is_scammer: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub is_spammer: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub is_underaged: bool,
    pub details: Option<String>,
}

impl AccountReportContent {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetCustomReportsConfigResult {
    pub config: Option<CustomReportsConfig>,
}
