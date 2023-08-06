use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use error_stack::ResultExt;

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir},
    ReadCommands,
};

use crate::{server::data::{database::{sqlite::SqliteSelectJson, diesel::DieselDatabaseError}, DatabaseError}, api::model::{AccountIdInternal, Profile, ProfileInternal}, utils::{IntoReportExt, IntoReportFromString}};

use error_stack::Result;

define_read_commands!(ReadCommandsProfile);

impl ReadCommandsProfile<'_> {
    pub async fn read_profile_directly_from_database(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, DatabaseError> {
        // return ProfileInternal::select_json(id, self.db())
        //     .await
        //     .change_context(DatabaseError::Sqlite);

        self.db_read(move |cmds| {
            cmds.profile().profile(id)
        }).await
    }
}
