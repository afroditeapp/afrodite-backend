use std::{collections::HashMap, fmt::Debug, net::IpAddr, sync::Arc};

use model::{AccountIdDb, IpAddressStorage, IpInfo};
use tokio::sync::Mutex;

struct State {
    accounts: HashMap<AccountIdDb, IpAddressStorage>,
}

pub struct IpAddressUsageTracker {
    state: Arc<Mutex<State>>,
}

impl Debug for IpAddressUsageTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IpAddressUsageTracker")
    }
}

impl Clone for IpAddressUsageTracker {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl IpAddressUsageTracker {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                accounts: HashMap::new(),
            }))
        }
    }

    pub async fn get_current_state_and_reset(&self) -> HashMap<AccountIdDb, IpAddressStorage> {
        let mut lock = self.state.lock().await;
        std::mem::take(&mut lock.accounts)
    }

    pub async fn mark_ip_used(
        &self,
        account: impl Into<AccountIdDb>,
        ip: IpAddr,
    ) {
        let account = account.into();
        let ip = ip.into();
        let mut lock = self.state.lock().await;
        {
            if let Some(storage) = lock.accounts.get_mut(&account) {
                if let Some(info) = storage.ips.get_mut(&ip) {
                    info.update_usage_info();
                } else {
                    storage.ips.insert(ip, IpInfo::new());
                }
                return;
            }
        }
        let storage = IpAddressStorage::new(ip);
        lock.accounts.insert(account, storage);
    }
}
