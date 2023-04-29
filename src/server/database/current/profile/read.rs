
use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};


use crate::server::database::current::SqliteReadCommands;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson};
use crate::api::account::data::AccountSetup;

use crate::api::model::{
    *
};

use crate::utils::{IntoReportExt};

use crate::read_json;


use std::collections::HashSet;


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
                public,
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
