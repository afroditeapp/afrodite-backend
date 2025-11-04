use std::{collections::HashSet, hash::Hash, sync::Arc};

use config::file::DemoAccountConfig;
use error_stack::Result;
use model::AccountId;
use model_server_state::{
    AccessibleAccountsInfo, DemoAccountId, DemoAccountLoginCredentials, DemoAccountLoginResult,
    DemoAccountToken,
};
use server_common::data::DataError;
use simple_backend_utils::{ContextExt, IntoReportFromString};
use tokio::sync::RwLock;

const HOUR_IN_SECONDS: u64 = 60 * 60;

#[derive(Debug, Clone)]
struct TokenState<T> {
    token: T,
    created: std::time::Instant,
}

impl<T: PartialEq + Clone> TokenState<T> {
    pub fn new(token: T) -> Self {
        Self {
            token,
            created: std::time::Instant::now(),
        }
    }

    pub fn get_checked(&self) -> Result<T, DataError> {
        if self.created.elapsed().as_secs() > HOUR_IN_SECONDS {
            Err("Token expired".to_string()).into_error_string(DataError::NotAllowed)
        } else {
            Ok(self.token.clone())
        }
    }
}

#[derive(Debug, Clone)]
struct DemoAccountAccountState {
    pub info: DemoAccountConfig,
    pub locked: bool,
    pub access_granted_token: Option<TokenState<DemoAccountToken>>,
}

impl DemoAccountAccountState {
    /// Returns error if token is expired.
    pub fn token_equals(&self, token: &DemoAccountToken) -> Result<bool, DataError> {
        if let Some(t) = &self.access_granted_token {
            Ok(&t.get_checked()? == token)
        } else {
            Ok(false)
        }
    }

    pub fn token_equals_unchecked(&self, token: &DemoAccountToken) -> bool {
        if let Some(t) = &self.access_granted_token {
            &t.token == token
        } else {
            false
        }
    }
}

#[derive(Debug)]
struct State {
    pub states: Vec<DemoAccountAccountState>,
}

#[derive(Debug)]
pub struct DemoAccountManager {
    state: Arc<RwLock<State>>,
}

impl Clone for DemoAccountManager {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl DemoAccountManager {
    pub fn new(info: Vec<DemoAccountConfig>) -> Result<Self, DataError> {
        let states: Vec<DemoAccountAccountState> = info
            .into_iter()
            .map(|info| DemoAccountAccountState {
                info,
                locked: false,
                access_granted_token: None,
            })
            .collect();

        fn check_uniqueness<T: Hash + Eq>(
            all_values: &mut HashSet<T>,
            value: T,
            create_error_text: impl FnOnce() -> String,
        ) -> Result<(), DataError> {
            if all_values.contains(&value) {
                return Err(create_error_text()).into_error_string(DataError::NotAllowed);
            }
            all_values.insert(value);

            Ok(())
        }

        let mut all_usernames = HashSet::<&str>::new();
        let mut all_passwords = HashSet::<&str>::new();
        let mut all_ids = HashSet::<i64>::new();
        for s in &states {
            check_uniqueness(&mut all_usernames, &s.info.username, || {
                format!("Duplicate username: {}", s.info.username)
            })?;
            check_uniqueness(&mut all_passwords, &s.info.password, || {
                format!("Duplicate password: {}", s.info.password)
            })?;
            check_uniqueness(&mut all_ids, s.info.database_id.id, || {
                format!("Duplicate database ID: {}", s.info.database_id.id)
            })?;
        }
        Ok(Self {
            state: Arc::new(RwLock::new(State { states })),
        })
    }

    pub async fn login(&self, credentials: DemoAccountLoginCredentials) -> DemoAccountLoginResult {
        let mut w = self.state.write().await;

        let i = {
            let Some((i, account)) = w
                .states
                .iter_mut()
                .enumerate()
                .find(|(_, v)| v.info.username == credentials.username)
            else {
                return DemoAccountLoginResult::default();
            };

            if account.info.password != credentials.password {
                account.locked = true;
                return DemoAccountLoginResult::default();
            }

            if account.locked {
                return DemoAccountLoginResult::locked();
            }

            i
        };

        // Make sure that token is unique.
        let token = loop {
            let token = DemoAccountToken::generate_new();
            if !w.states.iter().any(|s| s.token_equals_unchecked(&token)) {
                break TokenState::new(token);
            }
        };

        let account = w
            .states
            .get_mut(i)
            .expect("This should not happen. Index exists because Vec<DemoAccountAccountState> is not modified.");
        account.access_granted_token = Some(token.clone());
        DemoAccountLoginResult::token(token.token)
    }

    pub async fn valid_demo_account_token_exists(
        &self,
        token: &DemoAccountToken,
    ) -> Option<DemoAccountId> {
        let r = self.state.read().await;
        let account = r
            .states
            .iter()
            .find(|v| v.token_equals(token).unwrap_or(false));
        account.map(|v| v.info.database_id)
    }

    pub async fn demo_account_logout(&self, token: &DemoAccountToken) {
        let mut w = self.state.write().await;

        for a in &mut w.states {
            if a.token_equals_unchecked(token) {
                a.access_granted_token = None;
                break;
            }
        }
    }

    pub async fn accessible_accounts(
        &self,
        id: DemoAccountId,
    ) -> Result<AccessibleAccountsInfo, DataError> {
        let r = self.state.read().await;
        let state = r.states.iter().find(|state| state.info.database_id == id);

        if let Some(state) = state {
            if state.info.access_all_accounts {
                Ok(AccessibleAccountsInfo::All)
            } else {
                let accounts: Vec<AccountId> = state
                    .info
                    .accessible_accounts
                    .iter()
                    .map(|v| AccountId::new_base_64_url(*v))
                    .collect();
                Ok(AccessibleAccountsInfo::Specific {
                    config_file_accounts: accounts,
                    demo_account_id: state.info.database_id,
                })
            }
        } else {
            Err(DataError::NotFound.report())
        }
    }

    pub async fn max_account_count(&self, id: DemoAccountId) -> Result<u8, DataError> {
        let r = self.state.read().await;
        let state = r.states.iter().find(|state| state.info.database_id == id);

        if let Some(state) = state {
            Ok(state.info.max_account_count())
        } else {
            Err(DataError::NotFound.report())
        }
    }
}
