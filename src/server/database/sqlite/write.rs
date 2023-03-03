use error_stack::Result;

use crate::api::model::{Profile, AccountId};

use super::{SqliteDatabaseError, SqliteWriteHandle};

use crate::utils::IntoReportExt;

pub struct SqliteWriteCommands<'a> {
    handle: &'a SqliteWriteHandle,
}

impl<'a> SqliteWriteCommands<'a> {
    pub fn new(handle: &'a SqliteWriteHandle) -> Self {
        Self { handle }
    }

    pub async fn store_user_id(&mut self, user_id: &AccountId) -> Result<(), SqliteDatabaseError> {
        let id = user_id.as_str();
        sqlx::query!(
            r#"
            INSERT INTO User (id)
            VALUES (?)
            "#,
            id
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_user_profile(
        self,
        user_id: &AccountId,
        profile_data: &Profile,
    ) -> Result<(), SqliteDatabaseError> {
        let id = user_id.as_str();
        let name = profile_data.name();
        sqlx::query!(
            r#"
            UPDATE User
            SET name = ?
            WHERE id = ?
            "#,
            name,
            id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
