use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::api::{core::{user::{ApiKey, UserId}, profile::Profile}, self};

use super::database::{
    util::{DatabasePath, ProfileDirPath, WriteGuard},
    DatabaseOperationHandle,
};

use tracing::error;

pub struct SessionManager {
    api_keys: RwLock<HashMap<ApiKey, UserState>>,
    profiles: RwLock<HashMap<UserId, WriteGuard>>,
    database: DatabasePath,
    database_handle: DatabaseOperationHandle,
}

impl SessionManager {
    pub fn new(database: DatabasePath, database_handle: DatabaseOperationHandle) -> Self {
        let mut api_keys = HashMap::new();
        api_keys.insert(ApiKey::new("test".to_string()), UserState { profile: database.profile_dir("test") });

        Self {
            api_keys: RwLock::new(api_keys),
            profiles: RwLock::new(HashMap::new()),
            database,
            database_handle,
        }
    }

    /// New unique UUID is generated every time so no special handling needed.
    pub async fn register(&self) -> Result<UserId, ()> {
        let new_user_id = UserId::new(uuid::Uuid::new_v4().simple().to_string());
        let profile = self.database.profile_dir(new_user_id.as_str());
        let mut database = WriteGuard::new(profile, self.database_handle.clone());
        match database.write().register().await {
            Ok(()) => {
                self.profiles
                    .write()
                    .await
                    .insert(new_user_id.clone(), database);
                Ok(new_user_id)
            }
            Err(e) => {
                error!("Error: {e:?}");
                Err(())
            }
        }
    }

    pub async fn login(&self, user_id: UserId) -> Result<ApiKey, ()> {
        // TODO: check that UserId contains only hexadecimals

        if self.profiles.read().await.get(&user_id).is_none() {
            return Err(());
        }

        let token = ApiKey::new(uuid::Uuid::new_v4().simple().to_string());
        let user_state = UserState {
            profile: self.database.profile_dir(user_id.as_str()),
        };
        self.api_keys
            .write()
            .await
            .insert(token.clone(), user_state);

        // TODO: also save current api token to database
        Ok(token)
    }

    pub async fn get_profile(&self, user_id: UserId) -> Result<Profile, ()> {
        Ok(Profile::new("Name".to_string()))
    }

    pub async fn api_key_is_valid(&self, key: ApiKey) -> bool {
        self.api_keys.read().await.contains_key(&key)
    }
}

pub struct UserState {
    profile: ProfileDirPath,
}

impl UserState {}
