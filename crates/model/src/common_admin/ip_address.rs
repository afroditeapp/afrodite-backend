
use std::{collections::HashMap, net::{IpAddr, Ipv4Addr, Ipv6Addr}};

use diesel::{sql_types::Binary, Selectable};
use simple_backend_model::{diesel_bytes_try_from, UnixTime};

#[derive(Debug, Clone, Selectable)]
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
        Self {
            ips: map,
        }
    }
}
