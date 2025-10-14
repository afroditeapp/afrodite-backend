use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use diesel::{Selectable, prelude::Queryable, sql_types::Binary};
use serde::{Deserialize, Serialize};
use simple_backend_model::{IpCountryCounters, IpCountryKey, UnixTime, diesel_bytes_try_from};
use utoipa::ToSchema;

use crate::AccountId;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::ip_address_usage_statistics)]
#[diesel(check_for_backend(crate::Db))]
pub struct IpAddressInfoInternal {
    pub ip_address: IpAddressInternal,
    pub usage_count: i64,
    pub first_usage_unix_time: UnixTime,
    pub latest_usage_unix_time: UnixTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Binary)]
pub enum IpAddressInternal {
    V4([u8; 4]),
    V6([u8; 16]),
}

impl IpAddressInternal {
    pub fn to_ip_addr(&self) -> IpAddr {
        match *self {
            Self::V4(x) => IpAddr::V4(Into::<Ipv4Addr>::into(x)),
            Self::V6(x) => IpAddr::V6(Into::<Ipv6Addr>::into(x)),
        }
    }
}

diesel_bytes_try_from!(IpAddressInternal);

impl TryFrom<&[u8]> for IpAddressInternal {
    type Error = String;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if let Ok(v) = TryInto::<[u8; 4]>::try_into(value) {
            return Ok(Self::V4(v));
        }

        TryInto::<[u8; 16]>::try_into(value)
            .map(Self::V6)
            .map_err(|e| e.to_string())
    }
}

impl AsRef<[u8]> for IpAddressInternal {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::V4(v) => v,
            Self::V6(v) => v,
        }
    }
}

impl From<IpAddr> for IpAddressInternal {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(v) => Self::V4(v.octets()),
            IpAddr::V6(v) => Self::V6(v.octets()),
        }
    }
}

pub struct IpInfo {
    usage_count: i64,
    first_usage: UnixTime,
    latest_usage: UnixTime,
}

impl IpInfo {
    pub fn new() -> Self {
        let time = UnixTime::current_time();
        Self {
            usage_count: 1,
            first_usage: time,
            latest_usage: time,
        }
    }

    pub fn update_usage_info(&mut self) {
        self.usage_count += 1;
        self.latest_usage = UnixTime::current_time();
    }

    pub fn usage_count(&self) -> i64 {
        self.usage_count
    }

    pub fn first_usage(&self) -> UnixTime {
        self.first_usage
    }

    pub fn latest_usage(&self) -> UnixTime {
        self.latest_usage
    }
}

impl Default for IpInfo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IpAddressStorage {
    pub ips: HashMap<IpAddressInternal, IpInfo>,
}

impl IpAddressStorage {
    pub fn new(ip: IpAddressInternal) -> Self {
        let mut map = HashMap::new();
        map.insert(ip, IpInfo::new());
        Self { ips: map }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetIpAddressStatisticsSettings {
    pub account: AccountId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetIpAddressStatisticsResult {
    /// Latest used IP address is the first value.
    pub values: Vec<IpAddressInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct IpAddressInfo {
    /// IP address
    pub a: String,
    /// Usage count
    pub c: i64,
    /// First usage time
    pub f: UnixTime,
    /// Latest usage time
    pub l: UnixTime,
    /// IP list names. IP address belongs to these IP lists.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(default = json!([]))]
    pub lists: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum IpCountryStatisticsType {
    NewTcpConnections,
    NewHttpRequests,
}

/// Time range is inclusive. [Self::max_time] must be
/// greater or equal to [Self::min_time].
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetIpCountryStatisticsSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_time: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_time: Option<UnixTime>,
    pub statistics_type: IpCountryStatisticsType,
    /// Get statistics from RAM instead of database.
    pub data_from_ram: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct GetIpCountryStatisticsResult {
    pub values: Vec<IpCountryStatistics>,
}

impl GetIpCountryStatisticsResult {
    pub fn new_from_ip_country_tracker_state(
        data: HashMap<IpCountryKey, IpCountryCounters>,
        settings: GetIpCountryStatisticsSettings,
    ) -> Self {
        let values = data
            .into_iter()
            .filter_map(|(country, counters)| {
                let c = match settings.statistics_type {
                    IpCountryStatisticsType::NewTcpConnections => counters.tcp_connections(),
                    IpCountryStatisticsType::NewHttpRequests => counters.http_requests(),
                };

                if c == 0 {
                    return None;
                }

                let value = IpCountryStatisticsValue { t: None, c };

                Some(IpCountryStatistics {
                    country: country.to_ip_country().into_string(),
                    values: vec![value],
                })
            })
            .collect();

        Self { values }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct IpCountryStatistics {
    pub country: String,
    pub values: Vec<IpCountryStatisticsValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct IpCountryStatisticsValue {
    /// Value exists when [GetIpCountryStatisticsSettings::live_statistics] is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<UnixTime>,
    pub c: i64,
}
