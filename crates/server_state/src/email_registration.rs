use std::collections::HashMap;

use model::{AccountIdInternal, EmailLoginToken, EmailLoginTokenRow, UnixTime};
use model_server_data::EmailAddress;
use simple_backend_utils::time::DurationValue;
use tokio::sync::Mutex;

pub mod limit;

pub enum TokenData {
    Email(EmailAddress),
    Account(AccountIdInternal),
}

struct RegistrationToken {
    data: TokenData,
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

    fn cleanup_if_needed(&mut self, validity: DurationValue) {
        if self.last_cleanup.duration_value_elapsed(validity) {
            self.remove_expired(validity.seconds as i64);
            self.last_cleanup = UnixTime::current_time();
        }
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
        data: TokenData,
        validity: DurationValue,
    ) -> (EmailLoginToken, EmailLoginToken) {
        let mut lock = self.inner.lock().await;
        lock.cleanup_if_needed(validity);

        let (client_token, client_token_bytes) = loop {
            let (token, bytes) = EmailLoginToken::generate_new_with_bytes();
            if !lock.tokens.contains_key(&bytes) {
                break (token, bytes);
            }
        };
        let (email_token, email_token_bytes) = EmailLoginToken::generate_new_with_bytes();
        let unix_time = UnixTime::current_time();

        lock.tokens.insert(
            client_token_bytes,
            RegistrationToken {
                data,
                email_token: email_token_bytes,
                unix_time,
            },
        );

        (client_token, email_token)
    }

    /// Consume a token pair. Returns the data if valid.
    /// Removes the entry regardless of validity.
    pub async fn consume(
        &self,
        client_token: &[u8],
        email_token: &[u8],
        validity: DurationValue,
    ) -> Option<TokenData> {
        let mut lock = self.inner.lock().await;
        let entry = lock.tokens.remove(client_token)?;
        if entry.email_token == email_token && !entry.unix_time.duration_value_elapsed(validity) {
            Some(entry.data)
        } else {
            None
        }
    }

    /// Load all login tokens at once under a single lock.
    pub async fn load_all_login_tokens(
        &self,
        tokens: Vec<EmailLoginTokenRow>,
        validity: DurationValue,
    ) {
        let mut lock = self.inner.lock().await;
        for token in tokens {
            lock.tokens.insert(
                token.client_token.clone(),
                RegistrationToken {
                    data: TokenData::Account(token.account_id),
                    email_token: token.email_token,
                    unix_time: token.unix_time,
                },
            );
        }
        lock.cleanup_if_needed(validity);
    }

    /// Collect all valid login tokens and drain them from the store.
    /// Used during shutdown to persist tokens to DB.
    pub async fn drain_valid_login_tokens(
        &self,
        validity: DurationValue,
    ) -> Vec<EmailLoginTokenRow> {
        let mut lock = self.inner.lock().await;
        let now = UnixTime::current_time().ut;
        let max_age = validity.seconds as i64;

        lock.tokens
            .drain()
            .filter(|(_, token)| {
                matches!(token.data, TokenData::Account(_)) && now - token.unix_time.ut < max_age
            })
            .map(|(client_token_bytes, token)| {
                let account_id = match token.data {
                    TokenData::Account(id) => id,
                    _ => unreachable!(),
                };
                EmailLoginTokenRow {
                    account_id,
                    client_token: client_token_bytes,
                    email_token: token.email_token,
                    unix_time: token.unix_time,
                }
            })
            .collect()
    }
}
