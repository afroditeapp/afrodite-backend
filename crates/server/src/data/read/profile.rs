use error_stack::Result;

use model::{AccountIdInternal, ProfileInternal};

use crate::data::DatabaseError;

use super::{
    ReadCommands,
    super::{cache::DatabaseCache, file::utils::FileDir},
};

define_read_commands!(ReadCommandsProfile);

impl ReadCommandsProfile<'_> {
    pub async fn read_profile_directly_from_database(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, DatabaseError> {
        // return ProfileInternal::select_json(id, self.db())
        //     .await
        //     .change_context(DatabaseError::Sqlite);

        self.db_read(move |cmds| cmds.profile().profile(id)).await
    }
}
