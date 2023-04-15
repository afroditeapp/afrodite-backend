


use core::num;

use async_trait::async_trait;
use error_stack::Result;
use hyper::header::TRANSFER_ENCODING;
use sqlx::{Transaction, Sqlite};

use crate::{api::{
    account::data::AccountSetup,
    model::{Account, AccountIdInternal, Profile, AccountIdLight, ApiKey, NewModerationRequest, ContentId}, self, media::data::{ModerationRequestState, ModerationRequestId, ModerationRequestQueueNumber, ModerationId, ContentState, ContentIdInternal},
}, server::database::{sqlite::CurrentDataWriteHandle, file::file::ImageSlot}};

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

#[must_use]
pub struct DatabaseTransaction<'a> {
    transaction: Transaction<'a, Sqlite>,
}

impl <'a> DatabaseTransaction<'a> {
    pub async fn store_content_id_to_slot(
        handle: &'a CurrentDataWriteHandle,
        content_uploader: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<DatabaseTransaction<'a>, SqliteDatabaseError> {
        let content_uuid = content_id.as_uuid();
        let account_row_id = content_uploader.row_id();
        let state = ContentState::InSlot as i64;
        let slot = slot as i64;

        let mut transaction = handle.pool().try_begin().await.into_error(SqliteDatabaseError::TransactionBegin)?.ok_or(SqliteDatabaseError::TransactionBegin)?;

        sqlx::query!(
            r#"
            INSERT INTO MediaContent (content_id, account_row_id, moderation_state, slot_number)
            VALUES (?, ?, ?, ?)
            "#,
            content_uuid,
            account_row_id,
            state,
            slot,
        )
        .execute(&mut transaction)
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(DatabaseTransaction { transaction })
    }

    pub async fn commit(self) -> Result<(), SqliteDatabaseError> {
        self.transaction.commit().await.into_error(SqliteDatabaseError::TransactionCommit)
    }

    pub async fn rollback(self) -> Result<(), SqliteDatabaseError> {
        self.transaction.rollback().await.into_error(SqliteDatabaseError::TransactionRollback)
    }
}

pub struct DeletedSomething;

pub struct CurrentWriteMediaCommands<'a> {
    handle: &'a CurrentDataWriteHandle,
}

impl<'a> CurrentWriteMediaCommands<'a> {
    pub fn new(handle: &'a CurrentDataWriteHandle) -> Self {
        Self { handle }
    }

    async fn update_content_state(
        transaction: &mut Transaction<'_, Sqlite>,
        content_id: ContentId,
        new_state: ContentState,
    ) -> Result<(), SqliteDatabaseError> {
        let state = new_state as i64;

        sqlx::query!(
            r#"
            UPDATE MediaContent
            SET moderation_state = ?
            WHERE content_id = ?
            "#,
            state,
            content_id.content_id,
        )
        .execute(transaction)
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn store_content_id_to_slot(
        self,
        content_uploader: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<DatabaseTransaction<'a>, SqliteDatabaseError> {
        if !self.handle.read().media().get_content_id_from_slot(content_uploader, slot).await?.is_some() {
            return Err(SqliteDatabaseError::ContentSlotNotEmpty.into());
        }

        DatabaseTransaction::store_content_id_to_slot(self.handle, content_uploader, content_id, slot).await
    }

    pub async fn delete_image_from_slot(
        &self,
        request_creator: AccountIdInternal,
        slot: ImageSlot,
    ) -> Result<Option<DeletedSomething>, SqliteDatabaseError> {
        let account_row_id = request_creator.row_id();
        let in_slot_state = ContentState::InSlot as i64;
        let slot = slot as i64;
        let deleted_count = sqlx::query!(
            r#"
            DELETE FROM MediaContent
            WHERE account_row_id = ? AND moderation_state = ? AND slot_number = ?
            "#,
            account_row_id,
            in_slot_state,
            slot,
        )
        .execute(self.handle.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?
        .rows_affected();

        if deleted_count > 0 {
            Ok(Some(DeletedSomething))
        } else {
            Ok(None)
        }
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
    pub async fn create_new_moderation_request(
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

    pub async fn update_moderation_request(
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
            .get_moderation_request_content(target_id).await?;
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

    /// Update moderation state of Moderation.
    ///
    /// Also updates
    pub async fn update_moderation(
        &self,
        moderation_id: ModerationId,
        state: ModerationRequestState,
    ) -> Result<(), SqliteDatabaseError> {
        let content = self.handle.read().media().moderation(moderation_id).await?;

        let mut transaction = self.handle.pool().begin().await.into_error(SqliteDatabaseError::TransactionBegin)?;

        async fn actions(
            transaction: &mut Transaction<'_, Sqlite>,
            moderation: ModerationId,
            state: ModerationRequestState,
            content: NewModerationRequest,
        ) -> Result<(), SqliteDatabaseError> {
            let new_content_state = match state {
                ModerationRequestState::Accepted => ContentState::ModeratedAsAccepted,
                ModerationRequestState::Denied => ContentState::ModeratedAsDenied,
                ModerationRequestState::InProgress => ContentState::InModeration,
                ModerationRequestState::Waiting => ContentState::InSlot,
            };

            for c in content.content() {
                CurrentWriteMediaCommands::update_content_state(transaction, c, new_content_state).await?;
            }

            let state_number = state as i64;
            sqlx::query!(
                r#"
                UPDATE MediaModeration
                SET state_number = ?
                WHERE account_row_id = ? AND request_row_id = ?
                "#,
                state_number,
                moderation.account_id.account_row_id,
                moderation.request_id.request_row_id,
            )
            .execute(transaction)
            .await
            .into_error(SqliteDatabaseError::Execute)?;

            Ok(())
        }

        match actions(&mut transaction, moderation_id, state, content).await {
            Ok(()) =>  {
                transaction.commit().await.into_error(SqliteDatabaseError::TransactionCommit)
            }
            Err(e) => {
                match transaction.rollback().await.into_error(SqliteDatabaseError::TransactionRollback) {
                    Ok(()) => Err(e),
                    Err(another_error) => Err(another_error.attach(e)),
                }
            }
        }
    }
}
