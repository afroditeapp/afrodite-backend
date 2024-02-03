use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, Location, ProfileInternal};

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir},
    ReadCommands,
};
use crate::data::DataError;

define_read_commands!(ReadCommandsProfile);

impl ReadCommandsProfile<'_> {
    pub async fn profile(&self, id: AccountIdInternal) -> Result<ProfileInternal, DataError> {
        self.read_cache(id, move |cache| {
            cache.profile.as_ref().map(|data| data.data.clone())
        })
        .await?
        .ok_or(DataError::NotFound.report())
    }

    pub async fn profile_location(&self, id: AccountIdInternal) -> Result<Location, DataError> {
        self.db_read(move |mut cmds| cmds.profile().data().profile_location(id))
            .await
    }

    pub async fn read_profile_directly_from_database(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, DataError> {
        self.db_read(move |mut cmds| cmds.profile().data().profile(id))
            .await
    }

    pub async fn favorite_profiles(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| cmds.profile().favorite().favorites(id))
            .await
    }
}
