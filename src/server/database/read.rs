use crate::{
    api::core::user::{UserId, ApiKey},
    server::database::{DatabaseError}
};

/// Reading can be done async as Git library is not used.
pub struct DatabaseReadCommands<'a> {
    profile: &'a str,
}

impl<'a> DatabaseReadCommands<'a> {
    pub fn new(profile: &'a str) -> Self {
        Self { profile }
    }

    pub async fn api_key(&self, userId: UserId) -> String {
        unimplemented!()
    }

    pub async fn token(self) -> Result<ApiKey, DatabaseError> {
        unimplemented!()
    }
}
