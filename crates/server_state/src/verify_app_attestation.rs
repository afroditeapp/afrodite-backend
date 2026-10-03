use std::collections::HashMap;

use model::{AccountIdInternal, ClientType, UnixTime, VerifyAppAttestationToken};
use simple_backend_utils::time::DurationValue;
use tokio::sync::Mutex;

struct PendingLogin {
    account_id: AccountIdInternal,
    client_type: ClientType,
    connection_ip: std::net::IpAddr,
    unix_time: UnixTime,
}

struct StoreInner {
    tokens: HashMap<Vec<u8>, PendingLogin>,
    last_cleanup: UnixTime,
}

impl StoreInner {
    fn remove_expired(&mut self, max_age_seconds: i64) {
        let now = UnixTime::current_time().ut;
        self.tokens
            .retain(|_, token| now - token.unix_time.ut < max_age_seconds);
    }

    fn cleanup_if_needed(&mut self, validity: DurationValue) {
        if self.last_cleanup.duration_value_elapsed(validity) {
            self.remove_expired(validity.seconds as i64);
            self.last_cleanup = UnixTime::current_time();
        }
    }
}

/// In-memory store for pending logins that are waiting for app attestation
/// verification.
///
/// The login routes store a pending login here and return a short-lived
/// [VerifyAppAttestationToken]. The `post_verify_app_attestation` route
/// consumes the token and completes the login only if app attestation passes.
pub struct VerifyAppAttestationTokenStore {
    inner: Mutex<StoreInner>,
}

impl Default for VerifyAppAttestationTokenStore {
    fn default() -> Self {
        Self {
            inner: Mutex::new(StoreInner {
                tokens: HashMap::new(),
                last_cleanup: UnixTime::current_time(),
            }),
        }
    }
}

impl VerifyAppAttestationTokenStore {
    pub const TOKEN_VALIDITY: DurationValue = DurationValue::from_seconds(300);

    pub async fn insert(
        &self,
        account_id: AccountIdInternal,
        client_type: ClientType,
        connection_ip: std::net::IpAddr,
    ) -> VerifyAppAttestationToken {
        let mut lock = self.inner.lock().await;
        lock.cleanup_if_needed(Self::TOKEN_VALIDITY);

        let (token, bytes) = loop {
            let (token, bytes) = VerifyAppAttestationToken::generate_new_with_bytes();
            if !lock.tokens.contains_key(&bytes) {
                break (token, bytes);
            }
        };
        let unix_time = UnixTime::current_time();

        lock.tokens.insert(
            bytes,
            PendingLogin {
                account_id,
                client_type,
                connection_ip,
                unix_time,
            },
        );

        token
    }

    /// Consume a token. Returns the pending login data if the token is valid.
    /// Removes the entry regardless of validity.
    pub async fn consume(
        &self,
        token: &[u8],
    ) -> Option<(AccountIdInternal, ClientType, std::net::IpAddr)> {
        let mut lock = self.inner.lock().await;
        let entry = lock.tokens.remove(token)?;
        if !entry.unix_time.duration_value_elapsed(Self::TOKEN_VALIDITY) {
            Some((entry.account_id, entry.client_type, entry.connection_ip))
        } else {
            None
        }
    }
}
