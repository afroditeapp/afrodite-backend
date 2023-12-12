

use diesel::{
    prelude::*,
};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema};



#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct BackendConfig {
    pub bots: Option<BotConfig>
}

/// Enable automatic bots when server starts.
/// Editing of this field with edit module is only allowed when
/// this exists in the config file.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct BotConfig {
    /// User bot count
    pub users: u32,
    /// Admin bot count
    pub admins: u32,
}

// #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
// pub enum TimeGranularity {
//     Minutes,
//     Hours,
// }

// #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
// pub struct PerfHistoryQuery {
//     pub start_time: UnixTime,
//     pub end_time: UnixTime,
// }

// #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
// pub struct PerfValueArea {
//     /// Time for first data point in values.
//     pub start_time: UnixTime,
//     /// Time granularity for values in between start time and time points.
//     pub time_granularity: TimeGranularity,
//     pub values: Vec<u32>,
// }

// #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
// pub struct PerfHistoryValue {
//     pub counter_name: String,
//     pub values: Vec<PerfValueArea>,
// }

// #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
// pub struct PerfHistoryQueryResult {
//     pub counters: Vec<PerfHistoryValue>,
// }
