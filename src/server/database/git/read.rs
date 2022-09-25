use crate::{
    api::core::user::{ApiKey},
    server::database::{git::util::GitUserDirPath,
        git::file::CoreFileNoHistory, DatabaseError
    }
};

/// Reading can be done async as Git library is not used.
pub struct GitDatabaseReadCommands {
    profile: GitUserDirPath,
}

impl<'a> GitDatabaseReadCommands {
    pub fn new(profile: GitUserDirPath) -> Self {
        Self { profile }
    }

    pub async fn api_key(self) -> Result<Option<ApiKey>, DatabaseError> {
        let text = self.profile.read_to_string_optional(CoreFileNoHistory::ApiToken).await?;
        Ok(text.map(ApiKey::new))
    }
}
