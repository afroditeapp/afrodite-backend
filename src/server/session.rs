use std::collections::HashMap;

use tokio::sync::{Mutex, RwLock};

use crate::api::account::data::{ApiKey, AccountId};

use super::database::{write::WriteCommands, RouterDatabaseHandle};

pub struct SessionManager {
    /// Users which are logged in.
    pub api_keys: RwLock<HashMap<ApiKey, UserState>>,
    /// All users registered in the service.
    pub users: RwLock<HashMap<AccountId, Mutex<WriteCommands>>>,
    pub database: RouterDatabaseHandle,
}

impl SessionManager {
    pub async fn new(database_handle: RouterDatabaseHandle) -> Self {
        let mut api_keys = HashMap::new();
        let mut users = HashMap::new();
        //api_keys.insert(ApiKey::new("test".to_string()),
        // UserState { profile: database.profile_dir("test") });

        database_handle
            .read()
            .users(|user_id| {
                let write_commands = database_handle.user_write_commands(&user_id);
                users.insert(user_id, Mutex::new(write_commands));
            })
            .await
            .expect("User ID reading failed.");

        for id in users.keys() {
            let key = database_handle
                .read()
                .user_api_key(id)
                .await
                .expect("API key reading failed.");

            if let Some(key) = key {
                api_keys.insert(
                    key,
                    UserState {
                        user_id: id.clone(),
                    },
                );
            }
        }

        Self {
            api_keys: RwLock::new(api_keys),
            users: RwLock::new(users),
            database: database_handle,
        }
    }
}

pub struct UserState {
    user_id: AccountId,
}

impl UserState {
    pub fn new(user_id: AccountId) -> Self {
        Self { user_id }
    }

    pub fn id(&self) -> &AccountId {
        &self.user_id
    }
}
