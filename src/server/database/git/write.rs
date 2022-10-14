use std::{
    io::Write,
};

use tracing::error;

use crate::{
    server::database::{
        git::util::{GitUserDirPath},
        DatabaseError, GitDatabaseOperationHandle, git::file::{CoreFile, CoreFileNoHistory},
    }, api::core::{profile::Profile, user::{ApiKey}},
};

use super::{super::git::GitDatabase, GitError};

/// Make sure that you do not make concurrent writes.
pub struct GitDatabaseWriteCommands {
    profile: GitUserDirPath,
    /// This keeps database operation running even if quit singal is received.
    handle: GitDatabaseOperationHandle,
}

impl GitDatabaseWriteCommands {
    pub fn new(
        mut profile: GitUserDirPath,
        handle: GitDatabaseOperationHandle,
        common_message: Option<&str>,
    ) -> Self {
        profile.set_git_mode_message(common_message.map(|msg| msg.to_owned()));
        Self { profile, handle }
    }

    async fn run_git_command<
        T: FnOnce(GitUserDirPath) -> Result<(), DatabaseError> + Send + 'static,
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

    /// Create Git repository and store user id there.
    pub async fn store_user_id(self) -> Result<(), DatabaseError> {
        self.run_git_command(move |profile| {
            GitDatabase::create(&profile).map_err(DatabaseError::Git)?;

            profile.replace_file(CoreFile::Id, "Update user ID file", |file| {
                file.write_all(profile.id().as_str().as_bytes())
                    .map_err(|e| DatabaseError::Git(GitError::CreateIdFile(e)))
            })?;

            Ok(())
        }).await
    }

    pub async fn update_user_profile(self, profile_data: &Profile) -> Result<(), DatabaseError> {
        let profile_data = profile_data.clone();
        self.run_git_command(move |profile_dir| {
            profile_dir.replace_file(
                CoreFile::ProfileJson,
                "Update profile",
                move |file|
                    serde_json::to_writer(file, &profile_data)
                        .map_err(DatabaseError::Serialize),
            )
        }).await
    }

    pub async fn update_token(self, key: &ApiKey) -> Result<(), DatabaseError> {
        let key = key.clone();
        self.run_git_command(move |profile_dir| {
            profile_dir.replace_no_history_file(
                CoreFileNoHistory::ApiToken,
                move |file|
                    file.write_all(key.as_str().as_bytes())
                        .map_err(DatabaseError::FileIo),
            )
        }).await
    }

    pub async fn update_user_id(self) -> Result<(), DatabaseError> {
        self.run_git_command(move |profile| {
            profile.replace_file(CoreFile::Id, "Update user ID file", |file| {
                file.write_all(profile.id().as_str().as_bytes())
                    .map_err(|e| DatabaseError::Git(GitError::CreateIdFile(e)))
            })
        }).await
    }
}
