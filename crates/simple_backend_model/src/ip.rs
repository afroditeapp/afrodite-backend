use std::{
    borrow::Borrow,
    sync::atomic::{AtomicI64, Ordering},
};

/// Letter case is from IP country database for country codes
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IpCountryKeyRef<'a>(&'a str);

impl IpCountryKeyRef<'static> {
    pub const LOCALHOST: Self = Self("localhost");
    pub const UNKNOWN: Self = Self("unknown");
}

impl<'a> IpCountryKeyRef<'a> {
    pub fn new(country: &'a str) -> Self {
        Self(country)
    }

    pub fn to_ip_country_key(&self) -> IpCountryKey {
        IpCountryKey(self.0.to_string())
    }

    pub fn to_ip_country(&self) -> IpCountry {
        IpCountry::new(self.0)
    }

    pub fn as_str(&self) -> &str {
        self.0
    }
}

/// Letter case is from IP country database for country codes
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IpCountryKey(String);

impl IpCountryKey {
    pub fn new(country: &str) -> Self {
        Self(country.to_string())
    }

    pub fn to_ip_country(&self) -> IpCountry {
        IpCountry::new(&self.0)
    }
}

impl Borrow<str> for IpCountryKey {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

/// Lowercase IP country
///
/// # Possible values
///
/// - Two letter country code
/// - `localhost`
/// - `unknown`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IpCountry(String);

impl IpCountry {
    fn new(country: &str) -> Self {
        IpCountry(country.to_lowercase())
    }

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
