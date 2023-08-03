use error_stack::ResultExt;

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir},
    ReadCommands,
};

use crate::{server::data::{database::{current::SqliteReadCommands, sqlite::SqliteSelectJson}, DatabaseError}, api::model::{AccountIdInternal, Profile, ProfileInternal}};

use error_stack::Result;

define_read_commands!(ReadCommandsProfile);

impl ReadCommandsProfile<'_> {
    pub async fn read_profile_directly_from_database(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, DatabaseError> {
        ProfileInternal::select_json(id, self.db())
            .await
            .change_context(DatabaseError::Sqlite)
    }
}
