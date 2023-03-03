use error_stack::Result;

use crate::{
    api::model::{
        ApiKey, AccountId, Profile,
    },
    server::database::{git::file::CoreFileNoHistory, git::util::GitUserDirPath},
};

use super::{file::CoreFile, GitError};
use crate::utils::IntoReportExt;

/// Reading can be done async as Git library is not used.
pub struct GitDatabaseReadCommands {
    profile: GitUserDirPath,
}

impl<'a> GitDatabaseReadCommands {
    pub fn new(profile: GitUserDirPath) -> Self {
        Self { profile }
    }

    // Read user ID from file.
    pub async fn user_id(self) -> Result<Option<AccountId>, GitError> {
        let text = self.profile.read_to_string_optional(CoreFile::Id).await?;
        Ok(text.map(AccountId::new))
    }

    pub async fn api_key(self) -> Result<Option<ApiKey>, GitError> {
        let text = self
            .profile
            .read_to_string_optional(CoreFileNoHistory::ApiToken)
            .await?;
        Ok(text.map(ApiKey::new))
    }

    pub async fn profile(self) -> Result<Option<Profile>, GitError> {
        let text = self
            .profile
            .read_to_string_optional(CoreFile::ProfileJson)
            .await?;
        let profile = match text {
            None => return Ok(None),
            Some(text) => serde_json::from_str(&text).into_error(GitError::SerdeDerialize)?,
        };
        Ok(profile)
    }
}
