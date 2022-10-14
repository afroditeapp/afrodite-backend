use crate::{
    api::core::{user::{ApiKey, UserId}, profile::Profile},
    server::database::{git::util::GitUserDirPath,
        git::file::CoreFileNoHistory, DatabaseError
    }
};

use super::file::CoreFile;

/// Reading can be done async as Git library is not used.
pub struct GitDatabaseReadCommands {
    profile: GitUserDirPath,
}

impl<'a> GitDatabaseReadCommands {
    pub fn new(profile: GitUserDirPath) -> Self {
        Self { profile }
    }

    // Read user ID from file.
    pub async fn user_id(self) -> Result<Option<UserId>, DatabaseError> {
        let text = self.profile.read_to_string_optional(CoreFile::Id).await?;
        Ok(text.map(UserId::new))
    }

    pub async fn api_key(self) -> Result<Option<ApiKey>, DatabaseError> {
        let text = self.profile.read_to_string_optional(CoreFileNoHistory::ApiToken).await?;
        Ok(text.map(ApiKey::new))
    }

    pub async fn profile(self) -> Result<Option<Profile>, DatabaseError> {
        let text = self.profile.read_to_string_optional(CoreFile::ProfileJson).await?;
        let profile = match text {
            None => return Ok(None),
            Some(text) => serde_json::from_str(&text).map_err(DatabaseError::Derialize)?,
        };
        Ok(profile)
    }
}
