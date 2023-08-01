use crate::api::model::{AccountIdInternal, ProfileInternal};
use crate::server::data::database::current::SqliteReadCommands;
use crate::server::data::database::sqlite::{
    SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson,
};
use crate::server::data::index::location::LocationIndexKey;
use crate::utils::IntoReportExt;
use async_trait::async_trait;

pub struct CurrentReadProfileCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> CurrentReadProfileCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }
}

#[async_trait]
impl SqliteSelectJson for ProfileInternal {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> error_stack::Result<Self, SqliteDatabaseError> {
        let request = sqlx::query_as!(
            ProfileInternal,
            r#"
            SELECT
                version_uuid as "version_uuid: _",
                name,
                profile_text
            FROM Profile
            WHERE account_row_id = ?
            "#,
            id.account_row_id,
        )
        .fetch_one(read.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        Ok(request)
    }
}

#[async_trait]
impl SqliteSelectJson for LocationIndexKey {
    async fn select_json(
        id: AccountIdInternal,
        read: &SqliteReadCommands,
    ) -> error_stack::Result<Self, SqliteDatabaseError> {
        let request = sqlx::query_as!(
            LocationIndexKey,
            r#"
            SELECT
                location_key_x as "x: _",
                location_key_y as "y: _"
            FROM Profile
            WHERE account_row_id = ?
            "#,
            id.account_row_id,
        )
        .fetch_one(read.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        Ok(request)
    }
}
