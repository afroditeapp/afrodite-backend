

use api_client::models::new_moderation_request;
use async_trait::async_trait;
use error_stack::Result;
use tokio_stream::{Stream, StreamExt};

use super::super::super::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson};
use crate::api::account::data::AccountSetup;
use crate::api::media::data::{ModerationRequestState, ModerationRequestId, ModerationRequestQueueNumber, ModerationId, Content, ContentState};
use crate::api::model::{Account, AccountId, AccountIdInternal, ApiKey, Profile, ModerationRequest, NewModerationRequest, ContentId};
use crate::server::database::file::file::ImageSlot;
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

    pub async fn get_content_id_from_slot(
        &self,
        slot_owner: AccountIdInternal,
        slot: ImageSlot,
    ) -> Result<Option<ContentId>, SqliteDatabaseError> {
        let required_state = ContentState::InSlot as i64;
        let required_slot = slot as i64;
        let request = sqlx::query_as!(
            ContentId,
            r#"
            SELECT content_id as "content_id: _"
            FROM MediaContent
            WHERE account_row_id = ? AND moderation_state = ? AND slot_number = ?
            "#,
            slot_owner.account_row_id,
            required_state,
            required_slot,
        )
        .fetch_optional(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        Ok(request)
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

    pub async fn get_in_progress_moderations(
        &self,
        moderator_id: AccountIdInternal,
    ) -> Result<Vec<ModerationId>, SqliteDatabaseError> {
        let account_row_id = moderator_id.row_id();
        let state_accepted = ModerationRequestState::Accepted as i64;
        let state_denied = ModerationRequestState::Denied as i64;
        let data = sqlx::query!(
            r#"
            SELECT row_id
            FROM MediaModeration
            WHERE account_row_id = ? AND state_number != ? AND state_number != ?
            "#,
            account_row_id,
            state_accepted,
            state_denied,
        )
        .fetch_all(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?
        .into_iter().map(|r| ModerationId { moderation_row_id: r.row_id}).collect();

        Ok(data)
    }

    pub async fn get_next_active_queue_number(
        &self,
        sub_queue: i64,
    ) -> Result<Option<ModerationRequestQueueNumber>, SqliteDatabaseError> {
        let data = sqlx::query!(
            r#"
            SELECT queue_number
            FROM MediaModerationQueueNumber
            WHERE sub_queue = ?
            ORDER BY queue_number ASC
            LIMIT 1
            "#,
            sub_queue,
        )
        .fetch_optional(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Fetch)?;

        Ok(data.map(|r| ModerationRequestQueueNumber { number: r.queue_number}))
    }
}
