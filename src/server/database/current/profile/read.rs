use async_trait::async_trait;
use error_stack::Result;

use crate::server::database::current::SqliteReadCommands;
use crate::server::database::index::location::LocationIndexKey;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson};

use crate::api::model::*;

use crate::utils::IntoReportExt;

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
    ) -> Result<Self, SqliteDatabaseError> {
        let request = sqlx::query_as!(
            ProfileInternal,
            r#"
            SELECT
                image1 as "image1: _",
                image2 as "image2: _",
                image3 as "image3: _",
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
    ) -> Result<Self, SqliteDatabaseError> {
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
