


use core::num;

use async_trait::async_trait;
use error_stack::Result;

use crate::{api::{
    account::data::AccountSetup,
    model::{Account, AccountIdInternal, Profile, AccountIdLight, ApiKey, NewModerationRequest}, self, media::data::{ModerationRequestState, ModerationRequestId, ModerationRequestQueueNumber, ModerationId},
}, server::database::sqlite::CurrentDataWriteHandle};

use super::super::super::sqlite::{SqliteDatabaseError, SqliteUpdateJson, SqliteWriteHandle};

use crate::utils::IntoReportExt;

macro_rules! insert_or_update_json {
    ($self:expr, $sql:literal, $data:expr, $id:expr) => {{
        let id = $id.row_id();
        let data = serde_json::to_string($data).into_error(SqliteDatabaseError::SerdeSerialize)?;
        sqlx::query!($sql, data, id)
            .execute($self.handle.pool())
            .await
            .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }};
}

pub struct CurrentWriteMediaCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentWriteMediaCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    async fn delete_queue_number_of_account(
        &self,
        request_creator: AccountIdInternal,
    ) -> Result<(), SqliteDatabaseError> {
        let account_row_id = request_creator.row_id();
        sqlx::query!(
            r#"
            DELETE FROM MediaModerationQueueNumber
            WHERE account_row_id = ?
            "#,
            account_row_id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    async fn delete_queue_number(
        &self,
        number: ModerationRequestQueueNumber,
    ) -> Result<(), SqliteDatabaseError> {
        sqlx::query!(
            r#"
            DELETE FROM MediaModerationQueueNumber
            WHERE queue_number = ?
            "#,
            number.number,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn delete_moderation_request(
        &self,
        request_creator: AccountIdInternal,
    ) -> Result<(), SqliteDatabaseError> {
        // Delete old queue number and request

        self.delete_queue_number_of_account(request_creator).await?;
        let account_row_id = request_creator.row_id();

        sqlx::query!(
            r#"
            DELETE FROM MediaModerationRequest
            WHERE account_row_id = ?
            "#,
            account_row_id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        // Foreign key constraint removes MediaModeration rows.
        // Old data is not needed in current data database.

        Ok(())
    }



    /// Used when a user creates a new moderation request
    async fn create_new_moderation_request_queue_number(
        &self,
        request_creator: AccountIdInternal,
    ) -> Result<ModerationRequestQueueNumber, SqliteDatabaseError> {
        let account_row_id = request_creator.row_id();
        let queue_number = sqlx::query!(
            r#"
            INSERT INTO MediaModerationQueueNumber (account_row_id, sub_queue)
            VALUES (?, ?)
            "#,
            account_row_id,
            0,                // TODO: set to correct queue, for example if premium account
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?
        .last_insert_rowid();

        Ok(ModerationRequestQueueNumber {number: queue_number})
    }


    /// Used when a user creates a new moderation request
    pub async fn create_new_media_moderation_request(
        &self,
        request_creator: AccountIdInternal,
        request: NewModerationRequest,
    ) -> Result<(), SqliteDatabaseError> {
        // Delete old queue number and request
        self.delete_moderation_request(request_creator).await?;

        let account_row_id = request_creator.row_id();
        let queue_number = self.create_new_moderation_request_queue_number(request_creator).await?;
        let request_info = serde_json::to_string(&request).into_error(SqliteDatabaseError::SerdeSerialize)?;
        sqlx::query!(
            r#"
            INSERT INTO MediaModerationRequest (account_row_id, queue_number, json_text)
            VALUES (?, ?, ?)
            "#,
            account_row_id,
            queue_number.number,
            request_info,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_media_moderation_request(
        &self,
        request_owner_account_id: AccountIdInternal,
        new_request: NewModerationRequest,
    ) -> Result<(), SqliteDatabaseError> {
        // It does not matter if update is done even if moderation would be on
        // going.

        let request_info = serde_json::to_string(&new_request).into_error(SqliteDatabaseError::SerdeSerialize)?;
        let account_row_id = request_owner_account_id.row_id();

        sqlx::query!(
            r#"
            UPDATE MediaModerationRequest
            SET json_text = ?
            WHERE account_row_id = ?
            "#,
            request_info,
            account_row_id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn create_moderation(
        &self,
        target_id: ModerationRequestId,
        moderator_id: AccountIdInternal,
    ) -> Result<(), SqliteDatabaseError> {
        let (content, queue_number) = self.handle.read().media()
            .get_media_moderation_request_content(target_id).await?;
        let content_string = serde_json::to_string(&content)
            .into_error(SqliteDatabaseError::SerdeSerialize)?;
        let account_row_id = moderator_id.row_id();
        let state = ModerationRequestState::InProgress as i64;

        sqlx::query!(
            r#"
            INSERT INTO MediaModeration (account_row_id, request_row_id, state_number, json_text)
            VALUES (?, ?, ?, ?)
            "#,
            account_row_id,
            target_id.request_row_id,
            state,
            content_string,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        self.delete_queue_number(queue_number).await?;

        Ok(())
    }

    pub async fn update_moderation(
        &self,
        moderation: ModerationId,
        state: ModerationRequestState,
    ) -> Result<(), SqliteDatabaseError> {
        let state_number = state as i64;

        sqlx::query!(
            r#"
            UPDATE MediaModeration
            SET state_number = ?
            WHERE row_id = ?
            "#,
            state_number,
            moderation.moderation_row_id,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
