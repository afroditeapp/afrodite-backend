use error_stack::Result;
use serde::de::DeserializeOwned;

use crate::{
    api::model::{
        ApiKey, AccountId, Account, Profile, AccountSetup,
    },
    server::database::{file::file::CoreFileNoHistory, file::utils::GitUserDirPath},
};

use super::{file::{CoreFile, GetLiveVersionPath, GitJsonFile}, GitError};
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

    pub async fn read_json<T: DeserializeOwned + GitJsonFile>(
        self
    ) -> Result<Option<T>, GitError> {
        let text = self
            .account_dir
            .read_to_string_optional(T::FILE)
            .await?;
        let profile = match text {
            None => return Ok(None),
            Some(text) => serde_json::from_str(&text).into_error(GitError::SerdeDerialize)?,
        };
        Ok(profile)
    }
}
