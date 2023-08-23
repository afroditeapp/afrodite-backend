use error_stack::Result;
use model::{AccountIdInternal, ProfileInternal};

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir},
    ReadCommands,
};
use crate::data::DatabaseError;

define_read_commands!(ReadCommandsProfile);

impl ReadCommandsProfile<'_> {
    pub async fn profile(&self, id: AccountIdInternal) -> Result<ProfileInternal, DatabaseError> {
        self.read_cache(id, move |cache| {
            cache.profile.as_ref().map(|data| data.data.clone())
        })
        .await?
        .ok_or(DatabaseError::NotFound.into())
    }

    pub async fn read_profile_directly_from_database(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, DatabaseError> {
        self.db_read(move |cmds| cmds.profile().profile(id)).await
    }
}
