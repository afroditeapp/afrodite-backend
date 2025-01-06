use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::Text};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{diesel_string_wrapper, UnixTime};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub enum TimeGranularity {
    Minutes,
    Hours,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema, IntoParams)]
pub struct PerfMetricQuery {
    /// Start time for query results.
    pub start_time: Option<UnixTime>,
    /// End time for query results.
    pub end_time: Option<UnixTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct PerfMetricValueArea {
    /// Time for first data point in values.
    pub start_time: UnixTime,
    /// Time granularity for values in between start time and time points.
    pub time_granularity: TimeGranularity,
    pub values: Vec<u32>,
}

impl PerfMetricValueArea {
    pub fn average(&self) -> u32 {
        if self.values.is_empty() {
            return 0;
        }
        let sum: u64 = self.values.iter().map(|v| *v as u64).sum();
        (sum / self.values.len() as u64) as u32
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct PerfMetricValues {
    pub name: MetricName,
    pub values: Vec<PerfMetricValueArea>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct PerfMetricQueryResult {
    pub metrics: Vec<PerfMetricValues>,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct MetricKey {
    category: &'static str,
    name: &'static str,
}

impl MetricKey {
    const SYSTEM_CATEGORY: &str = "system";
    const SERVER_CATEGORY: &str = "server";

    pub const SYSTEM_CPU_USAGE: Self = Self {
        category: Self::SYSTEM_CATEGORY,
        name: "cpu_usage",
    };

    pub const SYSTEM_RAM_USAGE_MIB: Self = Self {
        category: Self::SYSTEM_CATEGORY,
        name: "ram_usage_mib",
    };

    pub const SERVER_WEBSOCKET_CONNECTIONS: Self = Self {
        category: Self::SERVER_CATEGORY,
        name: "websocket_connections",
    };

    pub fn new(category: &'static str, name: &'static str) -> MetricKey {
        Self {
            category,
            name,
        }
    }

    pub fn to_name(&self) -> MetricName {
        let name = format!("{}_{}", self.category, self.name);
        MetricName::new(name)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, Eq, PartialEq, ToSchema, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub struct MetricName(String);

impl MetricName {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

diesel_string_wrapper!(MetricName);
