use error_stack::Result;

use crate::api::model::{Profile, AccountId, AccountState};

use super::{SqliteDatabaseError, SqliteWriteHandle};

use crate::utils::IntoReportExt;

pub struct SqliteWriteCommands<'a> {
    handle: &'a SqliteWriteHandle,
}

impl<'a> SqliteWriteCommands<'a> {
    pub fn new(handle: &'a SqliteWriteHandle) -> Self {
        Self { handle }
    }

    pub async fn store_account_id(&mut self, id: &AccountId) -> Result<(), SqliteDatabaseError> {
        let id = id.as_str();
        sqlx::query!(
            r#"
            INSERT INTO Account (account_id)
            VALUES (?)
            "#,
            id
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_profile(
        self,
        user_id: &AccountId,
        profile_data: &Profile,
    ) -> Result<(), SqliteDatabaseError> {
        let id = user_id.as_str();
        let profile =
            serde_json::to_string(profile_data)
                .into_error(SqliteDatabaseError::SerdeSerialize)?;
        sqlx::query!(
            r#"
            UPDATE Profile
            SET profile_json = ?
            WHERE account_id = ?
            "#,
            profile,
            id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_account_state(
        self,
        id: &AccountId,
        account_state: &AccountState,
    ) -> Result<(), SqliteDatabaseError> {
        let id = id.as_str();
        let state =
            serde_json::to_string(account_state)
                .into_error(SqliteDatabaseError::SerdeSerialize)?;
        sqlx::query!(
            r#"
            UPDATE AccountState
            SET state_json = ?
            WHERE account_id = ?
            "#,
            state,
            id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
