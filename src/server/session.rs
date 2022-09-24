use std::collections::HashMap;

use tokio::sync::{RwLock, Mutex};

use crate::api::{core::{user::{ApiKey, UserId}, profile::Profile}, self};

use super::database::{
    write::WriteCommands, DatabaseManager, RouterDatabaseHandle,
};

use tracing::error;

pub struct SessionManager {
    /// Users which are logged in.
    pub api_keys: RwLock<HashMap<ApiKey, UserState>>,
    /// All users registered in the service.
    pub users: RwLock<HashMap<UserId, Mutex<WriteCommands>>>,
    pub database: RouterDatabaseHandle,
}

impl SessionManager {
    pub fn new(database_handle: RouterDatabaseHandle) -> Self {
        let mut api_keys = HashMap::new();
        //api_keys.insert(ApiKey::new("test".to_string()),
        // UserState { profile: database.profile_dir("test") });


        // TODO: load to ram here

        Self {
            api_keys: RwLock::new(api_keys),
            users: RwLock::new(HashMap::new()),
            database: database_handle,
        }
    }

    pub async fn get_profile(&self, user_id: UserId) -> Result<Profile, ()> {
        Ok(Profile::new("Name".to_string()))
    }

    pub async fn api_key_is_valid(&self, key: ApiKey) -> bool {
        self.api_keys.read().await.contains_key(&key)
    }
}

pub struct UserState {
    user_id: UserId,
}

impl UserState {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id
        }
    }
}
