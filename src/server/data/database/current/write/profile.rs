use async_trait::async_trait;
use crate::api::model::{AccountIdInternal, Profile, ProfileInternal, ProfileUpdateInternal};
use crate::server::data::database::current::CurrentDataWriteCommands;
use crate::server::data::database::sqlite::{CurrentDataWriteHandle, SqliteDatabaseError, SqliteSelectJson, SqliteUpdateJson};
use crate::server::data::index::location::LocationIndexKey;
use crate::server::data::write::WriteResult;
use crate::utils::IntoReportExt;

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
    ) -> WriteResult<ProfileInternal, SqliteDatabaseError, Profile> {
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
    ) -> error_stack::Result<(), SqliteDatabaseError> {
        sqlx::query!(
            r#"
            UPDATE Profile
            SET version_uuid = ?
            WHERE account_row_id = ?
            "#,
            self.version,
            id.account_row_id,
        )
        .execute(write.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}

#[async_trait]
impl SqliteUpdateJson for LocationIndexKey {
    async fn update_json(
        &self,
        id: AccountIdInternal,
        write: &CurrentDataWriteCommands,
    ) -> error_stack::Result<(), SqliteDatabaseError> {
        sqlx::query!(
            r#"
            UPDATE Profile
            SET location_key_x = ?, location_key_y = ?
            WHERE account_row_id = ?
            "#,
            self.x,
            self.y,
            id.account_row_id,
        )
        .execute(write.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
