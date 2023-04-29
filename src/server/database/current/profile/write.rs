


use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};


use crate::server::database::current::CurrentDataWriteCommands;
use crate::server::database::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson, SqliteUpdateJson, CurrentDataWriteHandle};
use crate::api::account::data::AccountSetup;

use crate::api::model::{
    *
};

use crate::utils::{IntoReportExt};

use crate::insert_or_update_json;


pub struct CurrentWriteProfileCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentWriteProfileCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    pub async fn init_profile(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, SqliteDatabaseError> {
        let version = uuid::Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO Profile (account_row_id, version_uuid)
            VALUES (?, ?)
            "#,
            id.account_row_id,
            version,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        let profile = ProfileInternal::select_json(id, &self.handle.read()).await?;
        Ok(profile)
    }
}


#[async_trait]
impl SqliteUpdateJson for ProfileUpdateInternal {
    async fn update_json(
        &self,
        id: AccountIdInternal,
        write: &CurrentDataWriteCommands,
    ) -> Result<(), SqliteDatabaseError> {
        sqlx::query!(
            r#"
            UPDATE Profile
            SET image1 = ?, image2 = ?, image3 = ?, version_uuid = ?
            WHERE account_row_id = ?
            "#,
            self.new_data.image1,
            self.new_data.image2,
            self.new_data.image3,
            self.version,
            id.account_row_id,
        )
        .execute(write.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
