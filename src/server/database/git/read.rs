use crate::{
    api::core::user::{UserId, ApiKey},
    server::database::{git::util::GitUserDirPath,
        git::file::CoreFileNoHistory, DatabaseError
    }
};

/// Reading can be done async as Git library is not used.
pub struct DatabaseReadCommands<'a> {
    profile: &'a GitUserDirPath,
}

impl<'a> DatabaseReadCommands<'a> {
    pub fn new(profile: &'a GitUserDirPath) -> Self {
        Self { profile }
    }

    pub async fn api_key(&self, user_id: UserId) -> String {
        "api_key".to_string()
    }

    pub async fn token(self) -> Result<ApiKey, DatabaseError> {
        let text = self.profile.read_to_string(CoreFileNoHistory::ApiToken).await?;
        Ok(ApiKey::new(text))
    }
}
