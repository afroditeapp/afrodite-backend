use std::{
    thread::sleep,
    time::Duration, io::Write,
};

use tracing::error;

use crate::{
    server::database::{
        util::{DatabasePath, ProfileDirPath},
        DatabaseError, DatabaseOperationHandle, file::{CoreFile, CoreFileNoHistory},
    }, api::core::{profile::Profile, user::UserApiToken},
};

use super::{super::git::GitDatabase};

pub struct DatabaseBasicCommands<'a> {
    database: &'a DatabasePath,
}

impl<'a> DatabaseBasicCommands<'a> {
    pub fn new(database: &'a DatabasePath) -> Self {
        Self { database }
    }
}

/// Make sure that you do not make concurrent writes.
pub struct DatabaseWriteCommands {
    profile: ProfileDirPath,
    /// This keeps database operation running even if quit singal is received.
    handle: DatabaseOperationHandle,
}

impl DatabaseWriteCommands {
    pub fn new(profile: ProfileDirPath, handle: DatabaseOperationHandle) -> Self {
        Self { profile, handle }
    }

    async fn run_command<
        T: FnOnce(ProfileDirPath) -> Result<(), DatabaseError> + Send + 'static,
    >(
        self,
        command: T,
    ) -> Result<(), DatabaseError> {
        let task = tokio::task::spawn_blocking(|| {
            let result = command(self.profile);
            drop(self.handle);
            result
        });

        // TODO: This might log user data here?
        let result = task.await.unwrap();
        if let Err(e) = &result {
            error!("Database write command error {e:?}");
        }
        result
    }

    pub async fn register(self) -> Result<(), DatabaseError> {
        self.run_command(move |profile| {
            GitDatabase::create(&profile).map_err(DatabaseError::Git)?;
            Ok(())
        }).await
    }

    pub async fn update_profile(self, profile_data: Profile) -> Result<(), DatabaseError> {
        self.run_command(move |profile_dir| {
            profile_dir.replace_file(
                CoreFile::ProfileJson,
                "Update profile",
                move |file|
                    serde_json::to_writer(file, &profile_data)
                        .map_err(DatabaseError::Serialize),
            )
        }).await
    }

    pub async fn update_token(self, token: UserApiToken) -> Result<(), DatabaseError> {
        self.run_command(move |profile_dir| {
            profile_dir.replace_no_history_file(
                CoreFileNoHistory::ApiToken,
                move |file|
                    file.write_all(token.as_bytes())
                        .map_err(DatabaseError::FileIo),
            )
        }).await
    }
}
