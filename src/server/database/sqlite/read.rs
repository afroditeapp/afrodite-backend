use error_stack::Result;
use tokio_stream::{Stream, StreamExt};

use super::{SqliteDatabaseError, SqliteReadHandle};
use crate::api::model::{Account, AccountId, Profile};
use crate::utils::IntoReportExt;

pub struct SqliteReadCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> SqliteReadCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub async fn profile(&self, id: &AccountId) -> Result<Profile, SqliteDatabaseError> {
        let id = id.as_str();
        let profile = sqlx::query!(
            r#"
            SELECT profile_json
            FROM Profile
            WHERE account_id = ?
            "#,
            id
        )
        .fetch_one(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        serde_json::from_str(&profile.profile_json)
            .into_error(SqliteDatabaseError::SerdeDeserialize)
    }

    pub async fn account_state(
        &self, id: &AccountId
    ) -> Result<Account, SqliteDatabaseError> {
        let id = id.as_str();
        let account = sqlx::query!(
            r#"
            SELECT state_json
            FROM AccountState
            WHERE account_id = ?
            "#,
            id
        )
        .fetch_one(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        serde_json::from_str(&account.state_json)
                .into_error(SqliteDatabaseError::SerdeDeserialize)
    }

    pub fn account_ids(&self) -> impl Stream<Item = Result<AccountId, SqliteDatabaseError>> + '_ {
        sqlx::query!(
            r#"
            SELECT account_id
            FROM Account
            "#,
        )
        .fetch(self.handle.pool())
        .map(|result| {
            let result = result
                .into_error(SqliteDatabaseError::Fetch)?;
            AccountId::parse(result.account_id)
                .into_error(SqliteDatabaseError::Fetch)
        })
    }

    // pub async fn users<T: FnMut(AccountId)>(&self, mut handle_user: T) -> impl Stream {
    //     let mut users = sqlx::query!(
    //         r#"
    //         SELECT id
    //         FROM User
    //         "#,
    //     )
    //     .fetch(self.handle.pool());

    //     while let Some(data) = users.try_next().await.map_err(SqliteDatabaseError::Execute)? {
    //         let id = AccountId::new(data.id);
    //         handle_user(id)
    //     }

    //     Ok(())
    // }
}
