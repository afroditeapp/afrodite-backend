use serde::{Deserialize, Serialize};
use simple_backend_model::UnixTime;
use utoipa::ToSchema;

use crate::AccountId;

/// Time range is inclusive. [Self::max_time] must be
/// greater or equal to [Self::min_time].
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetApiUsageStatisticsSettings {
    pub account: AccountId,
    pub max_time: Option<UnixTime>,
    pub min_time: Option<UnixTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetApiUsageStatisticsResult {
    pub values: Vec<ApiUsageStatistics>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ApiUsageStatistics {
    pub name: String,
    pub values: Vec<ApiUsageCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ApiUsageCount {
    pub t: UnixTime,
    pub c: i64,
}
