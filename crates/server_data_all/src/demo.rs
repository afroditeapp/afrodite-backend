use std::{collections::HashSet, sync::Arc};

use config::file::DemoModeConfig;
use error_stack::Result;
use model::{
    AccessibleAccount, AccountId, DemoModeConfirmLoginResult, DemoModeId, DemoModeLoginResult,
    DemoModeLoginToken, DemoModePassword, DemoModeToken,
};
use simple_backend_utils::{ContextExt, IntoReportFromString};
use tokio::sync::RwLock;
use tracing::error;

use crate::{
    app::{GetAccounts, GetConfig, ReadData},
    DataError,
};

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
struct DemoModeState {
    pub info: DemoModeConfig,
    pub locked: bool,
    pub stage1_token: Option<TokenState<DemoModeLoginToken>>,
    pub access_granted_token: Option<TokenState<DemoModeToken>>,
}

impl DemoModeState {
    /// Returns error if token is expired.
    pub fn stage1_token_equals(&self, token: &DemoModeLoginToken) -> Result<bool, DataError> {
        if let Some(t) = &self.stage1_token {
            Ok(&t.get_checked()? == token)
        } else {
            Ok(false)
        }
    }

    pub fn stage1_token_equals_unchecked(&self, token: &DemoModeLoginToken) -> bool {
        if let Some(t) = &self.stage1_token {
            &t.token == token
        } else {
            false
        }
    }

    /// Returns error if token is expired.
    pub fn token_equals(&self, token: &DemoModeToken) -> Result<bool, DataError> {
        if let Some(t) = &self.access_granted_token {
            Ok(&t.get_checked()? == token)
        } else {
            Ok(false)
        }
    }

    pub fn token_equals_unchecked(&self, token: &DemoModeToken) -> bool {
        if let Some(t) = &self.access_granted_token {
            &t.token == token
        } else {
            false
        }
    }
}

#[derive(Debug)]
struct State {
    pub states: Vec<DemoModeState>,
}

#[derive(Debug)]
pub struct DemoModeManager {
    state: Arc<RwLock<State>>,
}

impl Clone for DemoModeManager {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl DemoModeManager {
    pub fn new(info: Vec<DemoModeConfig>) -> Result<Self, DataError> {
        let states: Vec<DemoModeState> = info
            .into_iter()
            .map(|info| DemoModeState {
                info,
                locked: false,
                stage1_token: None,
                access_granted_token: None,
            })
            .collect();

        let check_password = |all_passwords: &mut HashSet<String>, password: &str| {
            if all_passwords.contains(password) {
                return Err(format!("Duplicate demo mode password: {}", password))
                    .into_error_string(DataError::NotAllowed);
            }
            all_passwords.insert(password.to_string());

            Ok(())
        };

        let mut all_passwords = HashSet::<String>::new();
        let mut all_ids = HashSet::<i64>::new();
        for s in &states {
            check_password(&mut all_passwords, &s.info.password_stage0)?;
            check_password(&mut all_passwords, &s.info.password_stage1)?;
            if all_ids.contains(&s.info.database_id.id) {
                return Err(format!("Duplicate database ID: {}", s.info.database_id.id))
                    .into_error_string(DataError::NotAllowed);
            }
            all_ids.insert(s.info.database_id.id);
        }
        Ok(Self {
            state: Arc::new(RwLock::new(State { states })),
        })
    }

    pub async fn stage0_login(
        &self,
        password: DemoModePassword,
    ) -> Result<DemoModeLoginResult, DataError> {
        let state_i = match self.stage0_password_exists(&password).await? {
            None => return Ok(DemoModeLoginResult::default()),
            Some(IndexOrLocked::Locked) => return Ok(DemoModeLoginResult::locked()),
            Some(IndexOrLocked::Index(i)) => i,
        };

        let mut w = self.state.write().await;
        // Make sure that token is unique.
        let token = loop {
            let token = DemoModeLoginToken::generate_new();
            if !w
                .states
                .iter()
                .any(|s| s.stage1_token_equals_unchecked(&token))
            {
                break TokenState::new(token);
            }
        };

        if let Some(s) = w.states.get_mut(state_i) {
            s.stage1_token = Some(token.clone());
            s.access_granted_token = None;
            Ok(DemoModeLoginResult::token(token.token))
        } else {
            Err(DataError::NotFound.report())
        }
    }

    async fn stage0_password_exists(
        &self,
        password: &DemoModePassword,
    ) -> Result<Option<IndexOrLocked>, DataError> {
        self.get_index_or_locked(|s| Ok(s.info.password_stage0 == password.password))
            .await
    }

    async fn stage1_password_exists(
        &self,
        password: &DemoModePassword,
    ) -> Result<Option<IndexOrLocked>, DataError> {
        self.get_index_or_locked(|s| Ok(s.info.password_stage1 == password.password))
            .await
    }

    async fn stage1_token_exists(
        &self,
        token: &DemoModeLoginToken,
    ) -> Result<Option<IndexOrLocked>, DataError> {
        self.get_index_or_locked(|s| s.stage1_token_equals(token))
            .await
    }

    pub async fn demo_mode_token_exists(
        &self,
        token: &DemoModeToken,
    ) -> Result<DemoModeId, DataError> {
        let result = self.get_index_or_locked(|s| s.token_equals(token)).await?;
        let token_index = match result {
            Some(IndexOrLocked::Index(i)) => i,
            Some(IndexOrLocked::Locked) => return Err(DataError::NotAllowed.report()),
            None => return Err(DataError::NotAllowed.report()),
        };

        if let Some(s) = self.state.read().await.states.get(token_index) {
            Ok(s.info.database_id)
        } else {
            Err(DataError::NotFound.report())
        }
    }

    pub async fn accessible_accounts_if_token_valid(
        &self,
        token: &DemoModeToken,
    ) -> Result<AccessibleAccountsInfo, DataError> {
        let r = self.state.read().await;
        let state =
            r.states
                .iter()
                .enumerate()
                .find(|(_, state)| match state.token_equals(token) {
                    Ok(true) => true,
                    Ok(false) | Err(_) => false,
                });

        if let Some((_, state)) = state {
            if state.info.access_all_accounts {
                Ok(AccessibleAccountsInfo::All)
            } else {
                let accounts: Vec<AccountId> = state
                    .info
                    .accessible_accounts
                    .iter()
                    .map(|v| AccountId::new(*v))
                    .collect();
                Ok(AccessibleAccountsInfo::Specific {
                    config_file_accounts: accounts,
                    demo_mode_id: state.info.database_id,
                })
            }
        } else {
            Err(DataError::NotAllowed.report())
        }
    }

    pub async fn stage1_login(
        &self,
        password: DemoModePassword,
        token: DemoModeLoginToken,
    ) -> Result<DemoModeConfirmLoginResult, DataError> {
        let token_i = match self.stage1_token_exists(&token).await? {
            None => return Ok(DemoModeConfirmLoginResult::default()),
            Some(IndexOrLocked::Locked) => return Ok(DemoModeConfirmLoginResult::locked()),
            Some(IndexOrLocked::Index(i)) => i,
        };

        let password_i = match self.stage1_password_exists(&password).await? {
            Some(IndexOrLocked::Locked) => return Ok(DemoModeConfirmLoginResult::locked()),
            Some(IndexOrLocked::Index(i)) => Some(i),
            None => None,
        };

        let mut w = self.state.write().await;

        if Some(token_i) != password_i {
            return if let Some(s) = w.states.get_mut(token_i) {
                // TODO: Print some more info like IP address? Store info that the
                // password needs to be changed.
                error!("Stage 1 password was wrong, locking the related demo mode credentials for index {}", token_i);
                s.locked = true;
                s.stage1_token = None;
                s.access_granted_token = None;
                Ok(DemoModeConfirmLoginResult::locked())
            } else {
                Err(DataError::NotFound.report())
            };
        }

        // Make sure that token is unique.
        let token = loop {
            let token = DemoModeToken::generate_new();
            if !w.states.iter().any(|s| s.token_equals_unchecked(&token)) {
                break TokenState::new(token);
            }
        };

        if let Some(s) = w.states.get_mut(token_i) {
            s.stage1_token = None;
            s.access_granted_token = Some(token.clone());
            Ok(DemoModeConfirmLoginResult::token(token.token))
        } else {
            Err(DataError::NotFound.report())
        }
    }

    async fn get_index_or_locked(
        &self,
        get_action: impl Fn(&DemoModeState) -> Result<bool, DataError>,
    ) -> Result<Option<IndexOrLocked>, DataError> {
        let r = self.state.read().await;
        let state = r.states.iter().enumerate().find_map(|(i, state)| {
            let result = get_action(state);
            match result {
                Ok(false) => None,
                Ok(true) if state.locked => Some(IndexOrLockedOrErr::Locked),
                Ok(true) => Some(IndexOrLockedOrErr::Index(i)),
                Err(e) => Some(IndexOrLockedOrErr::Error(e)),
            }
        });

        match state {
            Some(s) => Ok(Some(s.into_simple()?)),
            None => Ok(None),
        }
    }
}

enum IndexOrLockedOrErr {
    Locked,
    Index(usize),
    Error(error_stack::Report<DataError>),
}

impl IndexOrLockedOrErr {
    pub fn into_simple(self) -> Result<IndexOrLocked, DataError> {
        match self {
            IndexOrLockedOrErr::Locked => Ok(IndexOrLocked::Locked),
            IndexOrLockedOrErr::Index(i) => Ok(IndexOrLocked::Index(i)),
            IndexOrLockedOrErr::Error(e) => Err(e),
        }
    }
}

enum IndexOrLocked {
    Locked,
    Index(usize),
}

pub enum AccessibleAccountsInfo {
    All,
    Specific {
        config_file_accounts: Vec<AccountId>,
        demo_mode_id: DemoModeId,
    },
}

impl AccessibleAccountsInfo {
    pub async fn into_accounts<S: ReadData>(
        self,
        state: &S,
    ) -> crate::result::Result<Vec<AccountId>, DataError> {
        let (accounts, demo_mode_id) = match self {
            AccessibleAccountsInfo::All => {
                let all_accounts = state.read().account().account_ids_vec().await?;
                return Ok(all_accounts);
            }
            AccessibleAccountsInfo::Specific {
                config_file_accounts,
                demo_mode_id,
            } => (config_file_accounts, demo_mode_id),
        };

        let related_accounts = state
            .read()
            .account()
            .demo_mode_related_account_ids(demo_mode_id)
            .await?;

        Ok(accounts
            .into_iter()
            .chain(related_accounts.into_iter())
            .collect())
    }

    pub async fn with_extra_info<S: ReadData + GetConfig + GetAccounts>(
        self,
        state: &S,
    ) -> crate::result::Result<Vec<AccessibleAccount>, DataError> {
        let accounts = self.into_accounts(state).await?;

        let mut accessible_accounts = vec![];
        for id in &accounts {
            let info = if state.config().components().profile {
                let internal_id = state.get_internal_id(*id).await?;
                let profile = state.read().profile().profile(internal_id).await?;
                AccessibleAccount {
                    id: *id,
                    name: Some(profile.name),
                    age: Some(profile.age),
                }
            } else {
                AccessibleAccount {
                    id: *id,
                    name: None,
                    age: None,
                }
            };
            accessible_accounts.push(info);
        }

        Ok(accessible_accounts)
    }

    pub async fn contains<S: ReadData>(
        &self,
        account: AccountId,
        state: &S,
    ) -> crate::result::Result<(), DataError> {
        let (accounts, demo_mode_id) = match self {
            AccessibleAccountsInfo::All => return Ok(()),
            AccessibleAccountsInfo::Specific {
                config_file_accounts,
                demo_mode_id,
            } => (config_file_accounts, demo_mode_id),
        };

        let related_accounts = state
            .read()
            .account()
            .demo_mode_related_account_ids(*demo_mode_id)
            .await?;

        accounts
            .iter()
            .chain(related_accounts.iter())
            .find(|a| **a == account)
            .ok_or(DataError::NotFound.report())?;

        Ok(())
    }
}
