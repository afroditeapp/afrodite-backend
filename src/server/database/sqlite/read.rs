use tokio_stream::{StreamExt, Stream};

use crate::{server::database::DatabaseError, api::core::{profile::Profile, user::{ApiKey, UserId}}};

use super::{SqliteWriteHandle, SqliteDatabaseError, SqliteReadHandle};




pub struct SqliteReadCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl <'a> SqliteReadCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub async fn user_profile(&self, user_id: &UserId) -> Result<Profile, DatabaseError> {
        let id = user_id.as_str();
        let profile = sqlx::query!(
            r#"
            SELECT name
            FROM User
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(self.handle.pool()).await.map_err(SqliteDatabaseError::Execute)?;

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
        .map(|result| result
            .map_err(SqliteDatabaseError::Execute)
            .map(|data| UserId::new(data.id))
        )
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
