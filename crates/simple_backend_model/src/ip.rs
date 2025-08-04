use std::{
    borrow::Borrow,
    sync::atomic::{AtomicI64, Ordering},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct IpCountry(pub String);

impl Borrow<str> for IpCountry {
    fn borrow(&self) -> &str {
        self.0.as_ref()
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
