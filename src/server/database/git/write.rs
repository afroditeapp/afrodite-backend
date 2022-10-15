use std::{
    io::Write,
};
use error_stack::{Result};
use tracing::error;

use crate::{
    server::database::{
        git::util::{GitUserDirPath},
        GitDatabaseOperationHandle, git::file::{CoreFile, CoreFileNoHistory},
    }, api::core::{profile::Profile, user::{ApiKey}},
};
use crate::utils::IntoReportExt;
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
        T: FnOnce(GitUserDirPath) -> Result<(), GitError> + Send + 'static,
    >(
        self,
        command: T,
    ) -> Result<(), GitError> {
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
    pub async fn store_user_id(self) -> Result<(), GitError> {
        self.run_git_command(move |profile| {
            GitDatabase::create(&profile)?;

            profile.replace_file(CoreFile::Id, "Update user ID file", |file| {
                file.write_all(profile.id().as_str().as_bytes())
                    .into_error(GitError::IoFileWrite)
            })?;

            Ok(())
        }).await
    }

    pub async fn update_user_profile(self, profile_data: &Profile) -> Result<(), GitError> {
        let profile_data = profile_data.clone();
        self.run_git_command(move |profile_dir| {
            profile_dir.replace_file(
                CoreFile::ProfileJson,
                "Update profile",
                move |file|
                    serde_json::to_writer(file, &profile_data)
                        .into_error(GitError::SerdeSerialize),
            )
        }).await
    }

    pub async fn update_token(self, key: &ApiKey) -> Result<(), GitError> {
        let key = key.clone();
        self.run_git_command(move |profile_dir| {
            profile_dir.replace_no_history_file(
                CoreFileNoHistory::ApiToken,
                move |file|
                    file.write_all(key.as_str().as_bytes())
                        .into_error(GitError::IoFileWrite),
            )
        }).await
    }

    pub async fn update_user_id(self) -> Result<(), GitError> {
        self.run_git_command(move |profile| {
            profile.replace_file(CoreFile::Id, "Update user ID file", |file| {
                file.write_all(profile.id().as_str().as_bytes())
                    .into_error(GitError::IoFileWrite)
            })
        }).await
    }
}
