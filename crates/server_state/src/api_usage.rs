use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use model::{AccountIdDb, ApiUsage};
use tokio::sync::RwLock;

struct State {
    pub api_usage: HashMap<AccountIdDb, ApiUsage>,
}

pub struct ApiUsageTracker {
    state: Arc<RwLock<State>>,
}

impl Debug for ApiUsageTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApiUsageTracker")
    }
}

impl Clone for ApiUsageTracker {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl ApiUsageTracker {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(State {
                api_usage: HashMap::new(),
            })),
        }
    }

    pub async fn get_current_state_and_reset(&self) -> HashMap<AccountIdDb, ApiUsage> {
        let mut lock = self.state.write().await;
        std::mem::take(&mut lock.api_usage)
    }

    pub async fn incr(
        &self,
        account: impl Into<AccountIdDb>,
        api_getter: impl FnOnce(&ApiUsage) -> &AtomicU32,
    ) {
        let account = account.into();
        {
            let lock = self.state.read().await;
            if let Some(value) = lock.api_usage.get(&account) {
                api_getter(value).fetch_add(1, Ordering::Relaxed);
                return;
            }
        }

        let mut lock = self.state.write().await;
        let usage = ApiUsage::default();
        api_getter(&usage).fetch_add(1, Ordering::Relaxed);
        lock.api_usage.insert(account, usage);
    }
}
