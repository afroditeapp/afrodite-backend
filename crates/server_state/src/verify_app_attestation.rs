use std::collections::HashMap;

use base64::Engine;
use model::{AccountIdInternal, ClientType, LoginMethod, LoginPhaseTwoToken, UnixTime};
use simple_backend_utils::time::DurationValue;
use tokio::sync::Mutex;
use utils::random_bytes::random_128_bits;

struct PendingLogin {
    account_id: AccountIdInternal,
    client_type: ClientType,
    login_method: LoginMethod,
    connection_ip: std::net::IpAddr,
    unix_time: UnixTime,
}

/// These expire when server restarts as Apple App Attest docs say that
/// same key ID and challenge should be used when retrying key attestation
/// after App Attest server unavailable error.
struct ChallengeEntry {
    challenge: String,
}

struct StoreInner {
    tokens: HashMap<Vec<u8>, PendingLogin>,
    challenges: HashMap<AccountIdInternal, ChallengeEntry>,
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
/// [LoginPhaseTwoToken]. The `post_verify_app_attestation` route
/// consumes the token and completes the login only if app attestation passes.
pub struct VerifyAppAttestationTokenStore {
    inner: Mutex<StoreInner>,
}

impl Default for VerifyAppAttestationTokenStore {
    fn default() -> Self {
        Self {
            inner: Mutex::new(StoreInner {
                tokens: HashMap::new(),
                challenges: HashMap::new(),
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
        login_method: LoginMethod,
        connection_ip: std::net::IpAddr,
    ) -> LoginPhaseTwoToken {
        let mut lock = self.inner.lock().await;
        lock.cleanup_if_needed(Self::TOKEN_VALIDITY);

        let (token, bytes) = loop {
            let (token, bytes) = LoginPhaseTwoToken::generate_new_with_bytes();
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
                login_method,
                connection_ip,
                unix_time,
            },
        );

        token
    }

    /// Consume a token. Returns the pending login data if the token is valid.
    /// Removes the [PendingLogin] regardless of validity. Challenge is stored
    /// separately and it is not removed.
    pub async fn consume(
        &self,
        token: &[u8],
    ) -> Option<(
        AccountIdInternal,
        ClientType,
        LoginMethod,
        std::net::IpAddr,
        Option<String>,
    )> {
        let mut lock = self.inner.lock().await;
        let entry = lock.tokens.remove(token)?;
        if !entry.unix_time.duration_value_elapsed(Self::TOKEN_VALIDITY) {
            Some((
                entry.account_id,
                entry.client_type,
                entry.login_method,
                entry.connection_ip,
                lock.challenges
                    .get(&entry.account_id)
                    .map(|e| e.challenge.clone()),
            ))
        } else {
            None
        }
    }

    pub async fn remove_challenge(&self, account_id: AccountIdInternal) {
        let mut lock = self.inner.lock().await;
        lock.challenges.remove(&account_id);
    }

    /// Generate and store a challenge for the pending login behind `token`.
    ///
    /// The `connection_ip` must match the IP address that initiated the login
    /// so the challenge cannot be requested from a different IP.
    ///
    /// When `request_challenge` is set, it will be compared with current challenge.
    /// If the challenges match the challenge is reused.
    ///
    /// Returns `None` if `token` is not a valid pending login token, the IP
    /// address does not match. Otherwise returns the challenge and the
    /// account ID of the pending login.
    pub async fn generate_challenge(
        &self,
        token: &[u8],
        connection_ip: std::net::IpAddr,
        request_challenge: Option<String>,
    ) -> Option<(String, AccountIdInternal)> {
        let mut lock = self.inner.lock().await;
        lock.cleanup_if_needed(Self::TOKEN_VALIDITY);

        let entry = lock.tokens.get_mut(token)?;
        if entry.unix_time.duration_value_elapsed(Self::TOKEN_VALIDITY) {
            lock.tokens.remove(token);
            return None;
        }
        if entry.connection_ip != connection_ip {
            return None;
        }
        let account_id = entry.account_id;

        let challenge = if let Some(challenge) = request_challenge
            && Some(&challenge) == lock.challenges.get(&account_id).map(|e| &e.challenge)
        {
            challenge
        } else {
            let challenge =
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_128_bits());
            lock.challenges.insert(
                account_id,
                ChallengeEntry {
                    challenge: challenge.clone(),
                },
            );
            challenge
        };

        Some((challenge, account_id))
    }
}
