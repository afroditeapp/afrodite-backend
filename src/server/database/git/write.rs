use error_stack::Result;
use serde::Serialize;
use std::fmt::Debug;
use std::io::Write;
use tracing::error;

use super::file::{GetLiveVersionPath, GetGitPath, GetReplaceMessage};
use super::{super::git::GitDatabase, GitError};
use crate::api::account::data::AccountSetup;
use crate::api::model::{Account, Profile};
use crate::utils::IntoReportExt;
use crate::{
    api::model::{ApiKey},
    server::database::{
        git::file::{CoreFile, CoreFileNoHistory},
        git::util::GitUserDirPath,
        GitDatabaseOperationHandle,
    },
};

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

    async fn run_git_command<T: FnOnce(GitUserDirPath) -> Result<(), GitError> + Send + 'static>(
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
    pub async fn store_account_id(self) -> Result<(), GitError> {
        self.run_git_command(move |profile| {
            GitDatabase::create(&profile)?;

            profile.replace_file(CoreFile::Id, "Update user ID file", |file| {
                file.write_all(profile.id().as_str().as_bytes())
                    .into_error(GitError::IoFileWrite)
            })?;

            Ok(())
        })
        .await
    }

    pub async fn update_account(self, account: &Account) -> Result<(), GitError> {
        self.update_json(account, CoreFile::AccountStateJson).await
    }

    pub async fn update_account_setup(self, data: &AccountSetup) -> Result<(), GitError> {
        self.update_json(data, CoreFile::AccountSetupJson).await
    }

    pub async fn update_user_profile(self, profile_data: &Profile) -> Result<(), GitError> {
        self.update_json(profile_data, CoreFile::ProfileJson).await
    }

    pub async fn update_token(self, key: &ApiKey) -> Result<(), GitError> {
        let key = key.clone();
        self.run_git_command(move |profile_dir| {
            profile_dir.replace_no_history_file(CoreFileNoHistory::ApiToken, move |file| {
                file.write_all(key.as_str().as_bytes())
                    .into_error(GitError::IoFileWrite)
            })
        })
        .await
    }

    pub async fn update_user_id(self) -> Result<(), GitError> {
        self.run_git_command(move |profile| {
            profile.replace_file(CoreFile::Id, "Update user ID file", |file| {
                file.write_all(profile.id().as_str().as_bytes())
                    .into_error(GitError::IoFileWrite)
            })
        })
        .await
    }

    async fn update_json<
        T: Serialize + Clone + Send + 'static,
        S: GetLiveVersionPath + GetGitPath + GetReplaceMessage + Debug + Copy + Send + 'static
    >(
        self, data: &T, file: S,
    ) -> Result<(), GitError> {
        let data = data.clone();
        self.run_git_command(move |dir| {
            dir.replace_file(
                file,
                file.commit_message_for_replace(),
                move |file| {
                    serde_json::to_writer(file, &data).into_error(GitError::SerdeSerialize)
                }
            )
        })
        .await
    }

}
