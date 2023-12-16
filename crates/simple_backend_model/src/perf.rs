use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::UnixTime;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub enum TimeGranularity {
    Minutes,
    Hours,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct PerfHistoryQuery {
    /// Start time for query results.
    pub start_time: Option<UnixTime>,
    /// End time for query results.
    pub end_time: Option<UnixTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct PerfValueArea {
    /// Time for first data point in values.
    pub start_time: UnixTime,
    /// Time granularity for values in between start time and time points.
    pub time_granularity: TimeGranularity,
    pub values: Vec<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct PerfHistoryValue {
    pub counter_name: String,
    pub values: Vec<PerfValueArea>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct PerfHistoryQueryResult {
    pub counters: Vec<PerfHistoryValue>,
}
