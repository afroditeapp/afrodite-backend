use error_stack::Result;

use crate::{
    api::model::{
        ApiKey, AccountId, Profile, Account,
    },
    server::database::{git::file::CoreFileNoHistory, git::util::GitUserDirPath},
};

use super::{file::CoreFile, GitError};
use crate::utils::IntoReportExt;

/// Reading can be done async as Git library is not used.
pub struct GitDatabaseReadCommands {
    account_dir: GitUserDirPath,
}

impl<'a> GitDatabaseReadCommands {
    pub fn new(account_dir: GitUserDirPath) -> Self {
        Self { account_dir }
    }

    // Read AccounId from file.
    pub async fn account_id(self) -> Result<Option<AccountId>, GitError> {
        let text = self.account_dir
            .read_to_string_optional(CoreFile::Id).await?;
        match text {
            Some(raw_account_id) => {
                AccountId::parse(raw_account_id)
                    .into_error(GitError::AccountIdParsing)
                    .map(Some)
            }
            None => Ok(None)
        }
    }

    pub async fn api_key(self) -> Result<Option<ApiKey>, GitError> {
        let text = self
            .account_dir
            .read_to_string_optional(CoreFileNoHistory::ApiToken)
            .await?;
        Ok(text.map(ApiKey::new))
    }

    pub async fn profile(self) -> Result<Option<Profile>, GitError> {
        let text = self
            .account_dir
            .read_to_string_optional(CoreFile::ProfileJson)
            .await?;
        let profile = match text {
            None => return Ok(None),
            Some(text) => serde_json::from_str(&text).into_error(GitError::SerdeDerialize)?,
        };
        Ok(profile)
    }

    pub async fn account_state(self) -> Result<Option<Account>, GitError> {
        let text = self
            .account_dir
            .read_to_string_optional(CoreFile::AccountStateJson)
            .await?;
        let profile = match text {
            None => return Ok(None),
            Some(text) => serde_json::from_str(&text).into_error(GitError::SerdeDerialize)?,
        };
        Ok(profile)
    }
}
