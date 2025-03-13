use model::{AccountId, CustomReportId, CustomReportsConfig};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateCustomReportBoolean {
    pub target: AccountId,
    pub custom_report_id: CustomReportId,
    pub value: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetCustomReportsConfigResult {
    pub config: Option<CustomReportsConfig>,
}
