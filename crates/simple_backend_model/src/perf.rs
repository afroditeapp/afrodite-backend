use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::Text};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{UnixTime, diesel_string_wrapper};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub enum TimeGranularity {
    Minutes,
    Hours,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct PerfMetricQuery {
    /// Max value for inclusive time range.
    pub max_time: Option<UnixTime>,
    /// Min value for inclusive time range.
    pub min_time: Option<UnixTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, ToSchema)]
pub struct PerfMetricValueArea {
    /// Time value for the first data point. Every next time value is
    /// increased with [Self::time_granularity].
    pub first_time_value: UnixTime,
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
    pub group: Option<String>,
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
    group: Option<&'static str>,
}

impl MetricKey {
    const SYSTEM_CATEGORY: &str = "system";
    const WEBSOCKET_CATEGORY: &str = "websocket";

    pub const SYSTEM_CPU_USAGE: Self = Self::system("cpu_usage", "cpu");
    pub const SYSTEM_RAM_USAGE_MIB: Self = Self::system("ram_usage_mib", "ram");

    pub const CONNECTIONS: Self = Self::websocket("connections");
    pub const CONNECTIONS_MEN: Self = Self::websocket("connections_men");
    pub const CONNECTIONS_WOMEN: Self = Self::websocket("connections_women");
    pub const CONNECTIONS_NONBINARIES: Self = Self::websocket("connections_nonbinaries");

    pub const BOT_CONNECTIONS: Self = Self::websocket("bot_connections");
    pub const BOT_CONNECTIONS_MEN: Self = Self::websocket("bot_connections_men");
    pub const BOT_CONNECTIONS_WOMEN: Self = Self::websocket("bot_connections_women");
    pub const BOT_CONNECTIONS_NONBINARIES: Self = Self::websocket("bot_connections_nonbinaries");

    const fn system(name: &'static str, group: &'static str) -> Self {
        Self {
            category: Self::SYSTEM_CATEGORY,
            name,
            group: Some(group),
        }
    }
    const fn websocket(name: &'static str) -> Self {
        Self {
            category: Self::WEBSOCKET_CATEGORY,
            name,
            group: Some("websocket"),
        }
    }

    pub fn new(category: &'static str, name: &'static str) -> MetricKey {
        Self {
            category,
            name,
            group: None,
        }
    }

    pub fn to_name(&self) -> MetricName {
        let name = format!("{}_{}", self.category, self.name);
        MetricName::new(name)
    }

    pub fn group(&self) -> Option<&'static str> {
        self.group
    }
}

#[derive(
    Debug, Clone, Deserialize, Serialize, Hash, Eq, PartialEq, ToSchema, FromSqlRow, AsExpression,
)]
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

impl TryFrom<String> for MetricName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl AsRef<str> for MetricName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

diesel_string_wrapper!(MetricName);
