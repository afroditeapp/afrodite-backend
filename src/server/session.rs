use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};

use crate::api::{
    account::data::{AccountId, ApiKey},
    model::{AccountIdInternal, AccountIdLight, AccountState},
};

use super::database::{write::WriteCommands, RouterDatabaseHandle};

pub struct SessionManager {
    /// Accounts which are logged in.
    pub api_keys: RwLock<HashMap<ApiKey, Arc<AccountStateInRam>>>,
    /// All accounts registered in the service.
    pub accounts: RwLock<HashMap<AccountIdLight, Arc<AccountStateInRam>>>,
    pub database: RouterDatabaseHandle,
}

impl SessionManager {
    pub async fn new(database_handle: RouterDatabaseHandle) -> Self {
        // Load users and api keys from database to memory.

        let mut accounts = HashMap::new();
        database_handle
            .read()
            .account_ids(|id| {
                let state = AccountStateInRam {
                    id
                };
                accounts.insert(id.as_light(), Arc::new(state));
            })
            .await
            .expect("Account ID reading failed.");

        let mut api_keys = HashMap::new();
        for state in accounts.values() {
            let key = database_handle
                .read()
                .user_api_key(state.id)
                .await
                .expect("API key reading failed.");

            if let Some(key) = key {
                api_keys.insert(
                    key,
                    state.clone(),
                );
            }
        }

        Self {
            api_keys: RwLock::new(api_keys),
            accounts: RwLock::new(accounts),
            database: database_handle,
        }
    }
}

pub struct AccountStateInRam {
    id: AccountIdInternal,
}

impl AccountStateInRam {
    pub fn new(id: AccountIdInternal) -> Self {
        Self { id }
    }

    pub fn id(&self) -> AccountIdInternal {
        self.id
    }

    pub fn id_light(&self) -> AccountIdLight {
        self.id.as_light()
    }
}
