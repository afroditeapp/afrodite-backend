use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use error_stack::ResultExt;

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir},
    ReadCommands,
};

use crate::{server::data::{database::{current::SqliteReadCommands, sqlite::SqliteSelectJson}, DatabaseError}, api::model::{AccountIdInternal, Profile, ProfileInternal}, utils::{IntoReportExt, IntoReportFromString}};

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

        // let mut locked_connection = self.db()
        //     .handle
        //     .mutex
        //     .as_ref()
        //     .unwrap()
        //     .lock()
        //     .await;

        let connection = self.db()
            .handle
            .diesel_pool
            .get()
            .await
            .into_error(DatabaseError::Sqlite)?;
        let p = connection.interact(move |connection| {
            use crate::schema::Profile::dsl::*;

            let p: std::result::Result<crate::models::Profile, _> = Profile
                .filter(account_row_id.eq(id.row_id()))
                .first(connection);
            p
        })
            .await
            .into_error_string(DatabaseError::Sqlite)?
            .into_error_string(DatabaseError::Sqlite)?;

        // let connection: &mut SqliteConnection = &mut locked_connection;

        // use crate::schema::Profile::dsl::*;

        // let p: crate::models::Profile = Profile
        //     .filter(account_row_id.eq(id.row_id()))
        //     .first(connection)
        //     .unwrap();

        Ok(ProfileInternal { name: p.name, profile_text: p.profile_text,
        version_uuid: p.version_uuid })

    }
}
