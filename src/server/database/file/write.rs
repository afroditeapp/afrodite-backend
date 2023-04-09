use error_stack::Result;
use serde::Serialize;

use std::io::Write;
use tracing::error;

use super::file::{};
use super::{super::file::GitDatabase, GitError};

use crate::utils::IntoReportExt;
use crate::{
    api::model::ApiKey,
    server::database::{
        file::file::{},
        file::utils::AccountFilesDir,
        GitDatabaseOperationHandle,
    },
};

/// Make sure that you do not make concurrent writes.
pub struct GitDatabaseWriteCommands {
    profile: AccountFilesDir,
    /// This keeps database operation running even if quit singal is received.
    handle: GitDatabaseOperationHandle,
}

impl GitDatabaseWriteCommands {
    pub fn new(
        mut profile: AccountFilesDir,
        handle: GitDatabaseOperationHandle,
        common_message: Option<&str>,
    ) -> Self {

        Self { profile, handle }
    }

    async fn run_git_command<T: FnOnce(AccountFilesDir) -> Result<(), GitError> + Send + 'static>(
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

}
