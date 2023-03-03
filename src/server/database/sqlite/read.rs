use error_stack::Result;
use tokio_stream::{Stream, StreamExt};

use super::{SqliteDatabaseError, SqliteReadHandle};
use crate::api::model::{Profile, UserId};
use crate::utils::IntoReportExt;

pub struct SqliteReadCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> SqliteReadCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub async fn user_profile(&self, user_id: &UserId) -> Result<Profile, SqliteDatabaseError> {
        let id = user_id.as_str();
        let profile = sqlx::query!(
            r#"
            SELECT name
            FROM User
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(Profile::new(profile.name))
    }

    pub fn users(&self) -> impl Stream<Item = Result<UserId, SqliteDatabaseError>> + '_ {
        sqlx::query!(
            r#"
            SELECT id
            FROM User
            "#,
        )
        .fetch(self.handle.pool())
        .map(|result| {
            result
                .into_error(SqliteDatabaseError::Fetch)
                .map(|data| UserId::new(data.id))
        })
    }

    // pub async fn users<T: FnMut(UserId)>(&self, mut handle_user: T) -> impl Stream {
    //     let mut users = sqlx::query!(
    //         r#"
    //         SELECT id
    //         FROM User
    //         "#,
    //     )
    //     .fetch(self.handle.pool());

    //     while let Some(data) = users.try_next().await.map_err(SqliteDatabaseError::Execute)? {
    //         let id = UserId::new(data.id);
    //         handle_user(id)
    //     }

    //     Ok(())
    // }
}
