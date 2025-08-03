use std::{
    borrow::Borrow,
    collections::HashMap,
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
};

use tokio::sync::RwLock;

use crate::maxmind_db::MaxMindDbManagerData;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct IpCountry(pub String);

impl Borrow<str> for IpCountry {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Default, Debug)]
pub struct Counters {
    tcp_connections: AtomicI64,
    http_requests: AtomicI64,
}

impl Counters {
    pub fn tcp_connections(&self) -> i64 {
        self.tcp_connections.load(Ordering::Relaxed)
    }

    pub fn http_requests(&self) -> i64 {
        self.http_requests.load(Ordering::Relaxed)
    }

    pub(crate) fn increment_tcp_connections(&self) {
        self.tcp_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn increment_http_requests(&self) {
        self.http_requests.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Debug, Default)]
struct State {
    pub data: HashMap<IpCountry, Counters>,
}

pub struct IpCountryTracker {
    state: Arc<RwLock<State>>,
    ip_data: Arc<MaxMindDbManagerData>,
}

impl Clone for IpCountryTracker {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            ip_data: self.ip_data.clone(),
        }
    }
}

impl IpCountryTracker {
    pub(crate) fn new(ip_data: Arc<MaxMindDbManagerData>) -> Self {
        Self {
            state: Default::default(),
            ip_data,
        }
    }

    pub async fn get_current_state_and_reset(&self) -> HashMap<IpCountry, Counters> {
        let mut w = self.state.write().await;
        std::mem::take(&mut w.data)
    }

    pub async fn increment_tcp_connections(&self, ip: IpAddr) {
        self.track_internal(ip, |c| c.increment_tcp_connections())
            .await
    }

    pub async fn increment_http_requests(&self, ip: IpAddr) {
        self.track_internal(ip, |c| c.increment_http_requests())
            .await
    }

    async fn track_internal(&self, ip: IpAddr, action: impl FnOnce(&Counters)) {
        let ip_db = self.ip_data.current_db_ref().await;
        let Some(db) = ip_db.as_ref() else {
            return;
        };
        let country = db.get_country_ref(ip).unwrap_or("unknown");

        let r = self.state.read().await;
        if let Some(c) = r.data.get(country) {
            action(c);
            return;
        }

        let mut w = self.state.write().await;
        let counters = w.data.entry(IpCountry(country.to_string())).or_default();
        action(counters);
    }
}
