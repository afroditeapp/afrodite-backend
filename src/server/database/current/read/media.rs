

use api_client::models::new_moderation_request;
use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};

use super::super::super::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson};
use crate::api::account::data::AccountSetup;
use crate::api::media::data::{ModerationRequestState, ModerationRequestId, ModerationRequestQueueNumber};
use crate::api::model::{Account, AccountId, AccountIdInternal, ApiKey, Profile, ModerationRequest, NewModerationRequest};
use crate::server::database::read::ReadCmd;
use crate::server::database::utils::GetReadWriteCmd;
use crate::server::database::DatabaseError;
use crate::utils::{ErrorConversion, IntoReportExt};

macro_rules! read_json {
    ($self:expr, $id:expr, $sql:literal, $str_field:ident) => {{
        let id = $id.row_id();
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

pub struct CurrentReadMediaCommands<'a> {
    handle: &'a SqliteReadHandle,
}

impl<'a> CurrentReadMediaCommands<'a> {
    pub fn new(handle: &'a SqliteReadHandle) -> Self {
        Self { handle }
    }

    pub async fn current_media_moderation_request(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, SqliteDatabaseError> {
        let account_row_id = id.row_id();
        let request = sqlx::query!(
            r#"
            SELECT request_row_id, queue_number, json_text
            FROM MediaModerationRequest
            WHERE account_row_id = ?
            "#,
            account_row_id,
        )
        .fetch_optional(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let request = match request {
            None => return Ok(None),
            Some(r) => r,
        };


        let moderation_states = sqlx::query!(
            r#"
            SELECT state_number, json_text
            FROM MediaModeration
            WHERE request_row_id = ?
            "#,
            request.request_row_id,
        )
        .fetch_all(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let (state, data) = match moderation_states.first() {
            None => {
                (ModerationRequestState::Waiting, &request.json_text)
            }
            Some(first) => {
                let accepted = moderation_states.iter().find(|r| r.state_number == ModerationRequestState::Accepted as i64);
                let denied = moderation_states.iter().find(|r| r.state_number == ModerationRequestState::Denied as i64);

                if let Some(accepted) = accepted {
                    (ModerationRequestState::Accepted, &accepted.json_text)
                } else if let Some(denied) = denied {
                    (ModerationRequestState::Denied, &denied.json_text)
                } else {
                    (ModerationRequestState::InProgress, &first.json_text)
                }
            }
        };

        let data: NewModerationRequest = serde_json::from_str(data)
            .into_error(SqliteDatabaseError::SerdeDeserialize)?;

        Ok(Some(ModerationRequest::new(
            request.request_row_id,
            id.as_light(),
            state,
            data,
        )))

    }

    pub async fn get_media_moderation_request_content(
        &self,
        id: ModerationRequestId,
    ) -> Result<(NewModerationRequest, ModerationRequestQueueNumber), SqliteDatabaseError> {
        let request = sqlx::query!(
            r#"
            SELECT json_text, queue_number
            FROM MediaModerationRequest
            WHERE request_row_id = ?
            "#,
            id.request_row_id,
        )
        .fetch_one(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        let data: NewModerationRequest = serde_json::from_str(&request.json_text)
            .into_error(SqliteDatabaseError::SerdeDeserialize)?;

        Ok((data, ModerationRequestQueueNumber { number: request.queue_number}))
    }
}
