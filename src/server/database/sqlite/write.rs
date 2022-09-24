use crate::{server::database::DatabaseError, api::core::{profile::Profile, user::{ApiKey, UserId}}};

use super::{SqliteWriteHandle, SqliteDatabaseError};




pub struct SqliteWriteCommands<'a> {
    handle: &'a SqliteWriteHandle,
}

impl <'a> SqliteWriteCommands<'a> {
    pub fn new(handle: &'a SqliteWriteHandle) -> Self {
        Self { handle }
    }

    pub async fn store_user_id(&mut self, user_id: &UserId) -> Result<(), DatabaseError> {
        let id = user_id.as_str();
        sqlx::query!(
            r#"
            INSERT INTO User (id)
            VALUES (?)
            "#,
            id
        )
        .execute(self.handle.pool()).await.map_err(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_user_profile(self, user_id: &UserId, profile_data: &Profile) -> Result<(), DatabaseError> {
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
        .execute(self.handle.pool()).await.map_err(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
