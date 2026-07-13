use std::{collections::HashMap, net::IpAddr, sync::Arc};

use model::UnixTime;
use model_server_data::EmailAddress;
use simple_backend_utils::time::DurationValue;
use tokio::sync::Mutex;

struct RegistrationToken {
    email: EmailAddress,
    email_token: Vec<u8>,
    unix_time: UnixTime,
}

struct StoreInner {
    tokens: HashMap<Vec<u8>, RegistrationToken>,
    last_cleanup: UnixTime,
}

impl StoreInner {
    fn remove_expired(&mut self, max_age_seconds: i64) {
        let now = UnixTime::current_time().ut;
        self.tokens
            .retain(|_, token| now - token.unix_time.ut < max_age_seconds);
    }
}

pub struct EmailRegistrationTokenStore {
    inner: Mutex<StoreInner>,
}

impl Default for EmailRegistrationTokenStore {
    fn default() -> Self {
        Self {
            inner: Mutex::new(StoreInner {
                tokens: HashMap::new(),
                last_cleanup: UnixTime::current_time(),
            }),
        }
    }
}

impl EmailRegistrationTokenStore {
    pub async fn insert(
        &self,
        client_token: Vec<u8>,
        email_token: Vec<u8>,
        email: EmailAddress,
        unix_time: UnixTime,
        validity: DurationValue,
    ) {
        let mut lock = self.inner.lock().await;
        if lock.last_cleanup.duration_value_elapsed(validity) {
            lock.remove_expired(validity.seconds as i64);
            lock.last_cleanup = UnixTime::current_time();
        }
        lock.tokens.insert(
            client_token,
            RegistrationToken {
                email,
                email_token,
                unix_time,
            },
        );
    }

    /// Consume a token pair. Returns the email if valid.
    /// Removes the entry regardless of validity.
    pub async fn consume(
        &self,
        client_token: &[u8],
        email_token: &[u8],
        validity: DurationValue,
    ) -> Option<EmailAddress> {
        let mut lock = self.inner.lock().await;
        let entry = lock.tokens.remove(client_token)?;
        if entry.email_token == email_token && !entry.unix_time.duration_value_elapsed(validity) {
            Some(entry.email)
        } else {
            None
        }
    }
}

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
