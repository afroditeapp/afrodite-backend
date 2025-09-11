use std::{
    borrow::Borrow,
    sync::atomic::{AtomicI64, Ordering},
};

const LOCALHOST: &str = "localhost";
const UNKNOWN: &str = "unknown";

/// Letter case is from IP country database for country codes
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum IpCountryKeyRef<'a> {
    Country(&'a str),
    Localhost,
    Unknown,
}

impl<'a> IpCountryKeyRef<'a> {
    pub fn new(country: &'a str) -> Self {
        Self::Country(country)
    }

    pub fn to_ip_country_key(&self) -> IpCountryKey {
        match self {
            Self::Country(c) => IpCountryKey::Country(c.to_string()),
            Self::Localhost => IpCountryKey::Localhost,
            Self::Unknown => IpCountryKey::Unknown,
        }
    }

    pub fn to_ip_country(&self) -> IpCountry {
        self.to_ip_country_key().to_ip_country()
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Country(c) => c,
            Self::Localhost => LOCALHOST,
            Self::Unknown => UNKNOWN,
        }
    }
}

/// Letter case is from IP country database for country codes
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum IpCountryKey {
    Country(String),
    Localhost,
    Unknown,
}

impl IpCountryKey {
    pub fn to_ip_country(&self) -> IpCountry {
        match self {
            Self::Country(c) => IpCountry(c.to_uppercase()),
            Self::Localhost => IpCountry(LOCALHOST.to_string()),
            Self::Unknown => IpCountry(UNKNOWN.to_string()),
        }
    }
}

impl Borrow<str> for IpCountryKey {
    fn borrow(&self) -> &str {
        match self {
            Self::Country(c) => c.as_str(),
            Self::Localhost => LOCALHOST,
            Self::Unknown => UNKNOWN,
        }
    }
}

/// IP country
///
/// # Possible values
///
/// - Two letter uppercase country code
/// - `localhost`
/// - `unknown`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IpCountry(String);

impl IpCountry {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Default, Debug)]
pub struct IpCountryCounters {
    tcp_connections: AtomicI64,
    http_requests: AtomicI64,
}

impl IpCountryCounters {
    pub fn tcp_connections(&self) -> i64 {
        self.tcp_connections.load(Ordering::Relaxed)
    }

    pub fn http_requests(&self) -> i64 {
        self.http_requests.load(Ordering::Relaxed)
    }

    pub fn increment_tcp_connections(&self) {
        self.tcp_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_http_requests(&self) {
        self.http_requests.fetch_add(1, Ordering::Relaxed);
    }
}

impl Clone for IpCountryCounters {
    fn clone(&self) -> Self {
        Self {
            tcp_connections: AtomicI64::new(self.tcp_connections()),
            http_requests: AtomicI64::new(self.http_requests()),
        }
    }
}
