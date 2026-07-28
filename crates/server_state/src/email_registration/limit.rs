use std::{collections::HashMap, net::IpAddr, sync::Arc};

use model::UnixTime;
use simple_backend_utils::time::DurationValue;
use tokio::sync::Mutex;

struct RateLimiterInner {
    state: HashMap<IpAddr, u16>,
    last_cleanup: UnixTime,
}

pub struct EmailRegistrationRateLimiter {
    inner: Arc<Mutex<RateLimiterInner>>,
}

impl Default for EmailRegistrationRateLimiter {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RateLimiterInner {
                state: HashMap::new(),
                last_cleanup: UnixTime::current_time(),
            })),
        }
    }
}

impl Clone for EmailRegistrationRateLimiter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl EmailRegistrationRateLimiter {
    /// Returns `true` if the limit has been exceeded (i.e. the IP
    /// should be denied).
    pub async fn check_and_increment(&self, ip: IpAddr, max_per_day: u16) -> bool {
        let mut lock = self.inner.lock().await;
        if lock
            .last_cleanup
            .duration_value_elapsed(DurationValue::from_seconds(86400))
        {
            lock.state.clear();
            lock.last_cleanup = UnixTime::current_time();
        }

        let entry = lock.state.entry(ip).or_insert(0);
        if *entry >= max_per_day {
            true
        } else {
            *entry += 1;
            false
        }
    }
}
