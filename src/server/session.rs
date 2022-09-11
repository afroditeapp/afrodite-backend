use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::api::core::{user::{ApiKey, UserId}, profile::Profile};

use super::database::{
    util::{DatabasePath, ProfileDirPath, WriteGuard},
    DatabaseOperationHandle,
};

use tracing::error;

pub struct SessionManager {
    api_tokens: RwLock<HashMap<ApiKey, UserState>>,
    profiles: RwLock<HashMap<UserId, WriteGuard>>,
    database: DatabasePath,
    database_handle: DatabaseOperationHandle,
}

impl SessionManager {
    pub fn new(database: DatabasePath, database_handle: DatabaseOperationHandle) -> Self {
        Self {
            api_tokens: RwLock::new(HashMap::new()),
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
        self.api_tokens
            .write()
            .await
            .insert(token.clone(), user_state);

        // TODO: also save current api token to database
        Ok(token)
    }

    pub async fn get_profile(&self, user_id: UserId) -> Result<Profile, ()> {
        Err(())
    }
}

pub struct UserState {
    profile: ProfileDirPath,
}

impl UserState {}
