use std::collections::HashMap;

use tokio::sync::{Mutex, RwLock};

use crate::api::{account::data::{ApiKey, AccountId}, model::AccountIdLight};

use super::database::{write::WriteCommands, RouterDatabaseHandle};

pub struct SessionManager {
    /// Accounts which are logged in.
    pub api_keys: RwLock<HashMap<ApiKey, AccountStateInRam>>,
    /// All accounts registered in the service.
    pub accounts: RwLock<HashMap<AccountIdLight, Mutex<WriteCommands>>>,
    pub database: RouterDatabaseHandle,
}

impl SessionManager {
    pub async fn new(database_handle: RouterDatabaseHandle) -> Self {
        // Load users and api keys from database to memory.

        let mut accounts = HashMap::new();
        database_handle
            .read()
            .account_ids(|user_id| {
                let write_commands = database_handle.user_write_commands(user_id);
                accounts.insert(user_id, Mutex::new(write_commands));
            })
            .await
            .expect("User ID reading failed.");

        let mut api_keys = HashMap::new();
        for id in accounts.keys() {
            let id = &id.to_full();
            let key = database_handle
                .read()
                .user_api_key(id)
                .await
                .expect("API key reading failed.");

            if let Some(key) = key {
                api_keys.insert(
                    key,
                    AccountStateInRam {
                        user_id: id.clone(),
                    },
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
    user_id: AccountId,
}

impl AccountStateInRam {
    pub fn new(user_id: AccountId) -> Self {
        Self { user_id }
    }

    pub fn id(&self) -> &AccountId {
        &self.user_id
    }

    pub fn id_light(&self) -> AccountIdLight {
        self.user_id.as_light()
    }
}
