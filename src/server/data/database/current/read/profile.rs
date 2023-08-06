

use diesel::prelude::*;

use crate::api::model::{AccountIdInternal, ProfileInternal};
use crate::server::data::database::current::read::SqliteReadCommands;
use crate::server::data::database::sqlite::{
    SqliteDatabaseError, SqlxReadHandle, SqliteSelectJson,
};
use crate::server::data::index::location::LocationIndexKey;
use crate::server::data::read::ReadResult;
use crate::utils::IntoReportExt;
use async_trait::async_trait;
use crate::server::data::database::schema;


use diesel::{prelude::*, sqlite::Sqlite, deserialize::FromSql, sql_types::Binary, backend::Backend};

use crate::api::model::{AccountIdLight, ProfileVersion};



define_read_commands!(CurrentReadProfile, CurrentSyncReadProfile);




// #[derive(Queryable, Selectable, Debug)]
// #[diesel(table_name = schema::Profile)]
// #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// pub struct Profile {
//     account_row_id: i64,
//     pub version_uuid: ProfileVersion,
//     location_key_x: i64,
//     location_key_y: i64,
//     pub name: String,
//     pub profile_text: String,
// }


impl <'a> CurrentSyncReadProfile<'a> {
    pub fn profile(&'a mut self, id: AccountIdInternal) -> QueryResult<ProfileInternal> {
        use schema::Profile::dsl::*;

        Profile
            .filter(account_row_id.eq(id.account_row_id))
            .select(ProfileInternal::as_select())
            .first(self.conn())
    }

    pub fn location_index_key(&'a mut self, id: AccountIdInternal) -> QueryResult<LocationIndexKey> {
        use schema::Profile::dsl::*;

        let (x, y) = Profile
            .filter(account_row_id.eq(id.account_row_id))
            .select((location_key_x, location_key_y))
            .first::<(i64, i64)>(self.conn())?;

        Ok(LocationIndexKey { x: x as u16, y: y as u16 })
    }
}

#[async_trait]
impl SqliteSelectJson for ProfileInternal {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> error_stack::Result<Self, SqliteDatabaseError> {
        let request = sqlx::query!(
            r#"
            SELECT
                version_uuid as "version_uuid: uuid::Uuid",
                name,
                profile_text,
                location_key_x,
                location_key_y
            FROM Profile
            WHERE account_row_id = ?
            "#,
            id.account_row_id,
        )
        .fetch_one(read.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        Ok(ProfileInternal {
            account_row_id: id.account_row_id,
            version_uuid: ProfileVersion::new(request.version_uuid),
            location_key_x: 0,
            location_key_y: 0,
            name: request.name,
            profile_text: request.profile_text,
        })
    }
}

#[async_trait]
impl SqliteSelectJson for LocationIndexKey {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> error_stack::Result<Self, SqliteDatabaseError> {
       unimplemented!()
    }
}
