use crate::{api::core::user::{UserId, UserApiToken}, server::database::{util::ProfileDirPath, file::CoreFileNoHistory, DatabaseError}};

/// Reading can be done async as Git library is not used.
pub struct DatabaseReadCommands<'a> {
    profile: &'a ProfileDirPath,
}

impl<'a> DatabaseReadCommands<'a> {
    pub fn new(profile: &'a ProfileDirPath) -> Self {
        Self { profile }
    }

    pub async fn api_key(&self, userId: UserId) -> String {
        "api_key".to_string()
    }

    pub async fn token(self) -> Result<UserApiToken, DatabaseError> {
        self.profile.read_to_string(CoreFileNoHistory::ApiToken).await
    }
}
