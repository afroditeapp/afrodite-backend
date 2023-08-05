use error_stack::Result;
use time::OffsetDateTime;
use tokio_stream::{Stream, StreamExt};

use super::super::sqlite::{SqliteDatabaseError, SqlxReadHandle};
use super::HistoryData;

use crate::api::model::{AccountIdInternal, AccountState};

use crate::utils::IntoReportExt;

macro_rules! read_history {
    ($self:expr, $id:expr, $sql:literal, $str_field:ident) => {{
        let id = $id.as_uuid();
        sqlx::query!($sql, id)
            .fetch_one($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)
            .and_then(|data| {
                serde_json::from_str(&data.$str_field)
                    .into_error(SqliteDatabaseError::SerdeDeserialize)
            })
    }};
}

pub struct HistoryReadCommands<'a> {
    handle: &'a SqlxReadHandle,
}

impl<'a> HistoryReadCommands<'a> {
    pub fn new(handle: &'a SqlxReadHandle) -> Self {
        Self { handle }
    }

    pub async fn account_state_stream(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Stream<Item = Result<HistoryData<AccountState>, SqliteDatabaseError>> + '_ {
        #[derive(sqlx::FromRow)]
        struct HistoryAccount {
            row_id: i64,
            unix_time: i64,
            json_text: String,
        }

        sqlx::query_as::<_, HistoryAccount>(
            r#"
            SELECT row_id, unix_time, json_text
            FROM HistoryAccount
            WHERE account_row_id = ?
            "#,
        )
        .bind(account_id.row_id())
        .fetch(self.handle.pool())
        .map(move |result| {
            result
                .into_error(SqliteDatabaseError::Fetch)
                .and_then(|data| {
                    let value = serde_json::from_str(&data.json_text)
                        .into_error(SqliteDatabaseError::SerdeDeserialize)?;
                    let unix_time = OffsetDateTime::from_unix_timestamp(data.unix_time)
                        .into_error(SqliteDatabaseError::TimeParsing)?;
                    Ok(HistoryData {
                        row_id: data.row_id,
                        account_id: account_id.as_light(),
                        unix_time,
                        data: value,
                    })
                })
        })
    }
}
