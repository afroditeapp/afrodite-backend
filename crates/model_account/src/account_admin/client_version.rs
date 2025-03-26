use model::{ClientVersion, UnixTime};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Time range is inclusive. [Self::start_time] must be
/// greater or equal to [Self::end_time].
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetClientVersionStatisticsSettings {
    pub start_time: Option<UnixTime>,
    pub end_time: Option<UnixTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetClientVersionStatisticsResult {
    pub values: Vec<ClientVersionStatistics>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientVersionStatistics {
    pub version: ClientVersion,
    pub values: Vec<ClientVersionCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientVersionCount {
    pub t: UnixTime,
    pub c: i64,
}
