use crate::{server::database::util::ProfileDirPath, api::core::user::UserId};






/// Reading can be done async as Git library is not used.
pub struct DatabaseReadCommands<'a> {
    config: &'a ProfileDirPath,
}

impl <'a> DatabaseReadCommands<'a> {
    pub fn new(config: &'a ProfileDirPath) -> Self {
        Self { config }
    }

    pub async fn api_key(&self, userId: UserId) -> String {
        "api_key".to_string()
    }
}
