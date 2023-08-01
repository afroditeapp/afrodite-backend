use crate::server::data::database::sqlite::SqliteReadHandle;

pub struct CurrentReadChatCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> CurrentReadChatCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }
}

// #[async_trait]
// impl SqliteSelectJson for ProfileInternal {
//     async fn select_json(
//         id: AccountIdInternal,
//         read: &SqliteReadCommands,
//     ) -> Result<Self, SqliteDatabaseError> {
//         let request = sqlx::query_as!(
//             ProfileInternal,
//             r#"
//             SELECT
//                 version_uuid as "version_uuid: _",
//                 name,
//                 profile_text
//             FROM Profile
//             WHERE account_row_id = ?
//             "#,
//             id.account_row_id,
//         )
//         .fetch_one(read.handle.pool())
//         .await
//         .into_error(SqliteDatabaseError::Fetch)?;

//         Ok(request)
//     }
// }
