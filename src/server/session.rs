use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::api::core::user::{UserApiToken, UserId};

use super::database::{
    self,
    command::write::DatabaseWriteCommands,
    util::{DatabasePath, ProfileDirPath, WriteGuard},
    DatabaseOperationHandle,
};

use tracing::error;

pub struct SessionManager {
    api_tokens: RwLock<HashMap<UserApiToken, UserState>>,
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
        let new_user_id = uuid::Uuid::new_v4().simple().to_string();
        let profile = self.database.profile_dir(&new_user_id);
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

    pub async fn login(&self, user_id: UserId) -> Result<UserApiToken, ()> {
        // TODO: check that UserId contains only hexadecimals

        if self.profiles.read().await.get(&user_id).is_none() {
            return Err(());
        }

        let token = uuid::Uuid::new_v4().simple().to_string();
        let user_state = UserState {
            profile: self.database.profile_dir(&user_id),
        };
        self.api_tokens
            .write()
            .await
            .insert(token.clone(), user_state);

        // TODO: also save current api token to database
        Ok(token)
    }
}

pub struct UserState {
    profile: ProfileDirPath,
}

impl UserState {}
