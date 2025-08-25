use std::{collections::HashMap, net::IpAddr, sync::Arc};

use simple_backend_model::{IpCountryCounters, IpCountryKey, IpCountryKeyRef};
use tokio::sync::RwLock;

use crate::maxmind_db::MaxMindDbManagerData;

#[derive(Debug, Default)]
struct State {
    pub data: HashMap<IpCountryKey, IpCountryCounters>,
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

    pub async fn get_current_state_and_reset(&self) -> HashMap<IpCountryKey, IpCountryCounters> {
        let mut w = self.state.write().await;
        std::mem::take(&mut w.data)
    }

    pub async fn copy_current_state(&self) -> HashMap<IpCountryKey, IpCountryCounters> {
        let r = self.state.read().await;
        r.data.clone()
    }

    pub async fn increment_tcp_connections(&self, ip: IpAddr) {
        self.track_internal(ip, |c| c.increment_tcp_connections())
            .await
    }

    pub async fn increment_http_requests(&self, ip: IpAddr) {
        self.track_internal(ip, |c| c.increment_http_requests())
            .await
    }

    async fn track_internal(&self, ip: IpAddr, action: impl FnOnce(&IpCountryCounters)) {
        let ip_db = self.ip_data.current_db_ref().await;
        let Some(db) = ip_db.as_ref() else {
            return;
        };
        let country = if ip.is_loopback() {
            IpCountryKeyRef::LOCALHOST
        } else {
            db.get_country_ref(ip).unwrap_or(IpCountryKeyRef::UNKNOWN)
        };

        if let Some(c) = self.state.read().await.data.get(country.as_str()) {
            action(c);
            return;
        }

        let mut w = self.state.write().await;
        let counters = w.data.entry(country.to_ip_country_key()).or_default();
        action(counters);
    }
}
