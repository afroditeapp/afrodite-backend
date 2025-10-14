use model::{AccountId, CustomReportId, CustomReportsConfig};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateCustomReportEmpty {
    pub target: AccountId,
    pub custom_report_id: CustomReportId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetCustomReportsConfigResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<CustomReportsConfig>,
}
