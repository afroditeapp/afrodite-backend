use model::{
    AccountIdInternal, ContentId, ContentState, CurrentAccountMediaInternal, ModerationRequest,
    ModerationRequestContent, ModerationRequestQueueNumber, PrimaryImage, ImageSlot,
};
use crate::sqlite::SqliteDatabaseError;

use crate::WriteResult;
use utils::IntoReportExt;
use sqlx::{Sqlite, Transaction};

define_write_commands!(CurrentWriteMedia, CurrentSyncWriteMedia);

#[must_use]
pub struct DatabaseTransaction<'a> {
    transaction: Transaction<'a, Sqlite>,
}

impl<'a> DatabaseTransaction<'a> {
    pub async fn store_content_id_to_slot(
        pool: &'a sqlx::Pool<Sqlite>,
        content_uploader: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> error_stack::Result<DatabaseTransaction<'a>, SqliteDatabaseError> {
        let content_uuid = content_id.as_uuid();
        let account_row_id = content_uploader.row_id();
        let state = ContentState::InSlot as i64;
        let slot = slot as i64;

        let mut transaction = pool
            .begin()
            .await
            .into_error(SqliteDatabaseError::TransactionBegin)?;

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
        .execute(&mut *transaction)
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(DatabaseTransaction { transaction })
    }

    pub async fn commit(self) -> error_stack::Result<(), SqliteDatabaseError> {
        self.transaction
            .commit()
            .await
            .into_error(SqliteDatabaseError::TransactionCommit)
    }

    pub async fn rollback(self) -> error_stack::Result<(), SqliteDatabaseError> {
        self.transaction
            .rollback()
            .await
            .into_error(SqliteDatabaseError::TransactionRollback)
    }
}

pub struct DeletedSomething;

impl<'a> CurrentWriteMedia<'a> {
    pub async fn init_current_account_media(
        &self,
        id: AccountIdInternal,
    ) -> WriteResult<(), SqliteDatabaseError, CurrentAccountMediaInternal> {
        let _request = sqlx::query!(
            r#"
            INSERT INTO CurrentAccountMedia
                (account_row_id)
            VALUES (?)
            "#,
            id.account_row_id,
        )
        .execute(self.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_current_account_media_with_primary_image(
        &self,
        id: AccountIdInternal,
        primary_image: PrimaryImage,
    ) -> WriteResult<(), SqliteDatabaseError, CurrentAccountMediaInternal> {
        let _request = sqlx::query!(
            r#"
            UPDATE CurrentAccountMedia
            SET profile_content_row_id = ?,
            grid_crop_size = ?,
            grid_crop_x = ?,
            grid_crop_y = ?
            WHERE account_row_id = ?
            "#,
            primary_image.content_id,
            primary_image.grid_crop_size,
            primary_image.grid_crop_x,
            primary_image.grid_crop_y,
            id.account_row_id,
        )
        .execute(self.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn store_content_id_to_slot(
        self,
        content_uploader: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> error_stack::Result<DatabaseTransaction<'a>, SqliteDatabaseError> {
        if self
            .read()
            .media()
            .get_content_id_from_slot(content_uploader, slot)
            .await?
            .is_some()
        {
            return Err(SqliteDatabaseError::ContentSlotNotEmpty.into());
        }

        DatabaseTransaction::store_content_id_to_slot(
            self.pool(),
            content_uploader,
            content_id,
            slot,
        )
        .await
    }

    pub async fn delete_image_from_slot(
        &self,
        request_creator: AccountIdInternal,
        slot: ImageSlot,
    ) -> error_stack::Result<Option<DeletedSomething>, SqliteDatabaseError> {
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
        .execute(self.pool())
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
    ) -> error_stack::Result<(), SqliteDatabaseError> {
        let account_row_id = request_creator.row_id();
        sqlx::query!(
            r#"
            DELETE FROM MediaModerationQueueNumber
            WHERE account_row_id = ?
            "#,
            account_row_id,
        )
        .execute(self.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn delete_moderation_request(
        &self,
        request_creator: AccountIdInternal,
    ) -> error_stack::Result<(), SqliteDatabaseError> {
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
        .execute(self.pool())
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
    ) -> error_stack::Result<ModerationRequestQueueNumber, SqliteDatabaseError> {
        let account_row_id = request_creator.row_id();
        let queue_number = sqlx::query!(
            r#"
            INSERT INTO MediaModerationQueueNumber (account_row_id, sub_queue)
            VALUES (?, ?)
            "#,
            account_row_id,
            0, // TODO: set to correct queue, for example if premium account
        )
        .execute(self.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?
        .last_insert_rowid();

        Ok(ModerationRequestQueueNumber {
            number: queue_number,
        })
    }

    /// Used when a user creates a new moderation request
    ///
    /// Moderation request content must content ids which point to your own
    /// image slots. Otherwise this returns an error.
    pub async fn create_new_moderation_request(
        &self,
        request_creator: AccountIdInternal,
        request: ModerationRequestContent,
    ) -> WriteResult<(), SqliteDatabaseError, ModerationRequest> {
        self.read()
            .media()
            .content_validate_moderation_request_content(request_creator, &request)
            .await?;

        // Delete old queue number and request
        self.delete_moderation_request(request_creator).await?;

        let account_row_id = request_creator.row_id();
        let queue_number = self
            .create_new_moderation_request_queue_number(request_creator)
            .await?;
        let request_info =
            serde_json::to_string(&request).into_error(SqliteDatabaseError::SerdeSerialize)?;
        sqlx::query!(
            r#"
            INSERT INTO MediaModerationRequest (account_row_id, queue_number, json_text)
            VALUES (?, ?, ?)
            "#,
            account_row_id,
            queue_number.number,
            request_info,
        )
        .execute(self.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }

    pub async fn update_moderation_request(
        &self,
        request_owner_account_id: AccountIdInternal,
        new_request: ModerationRequestContent,
    ) -> error_stack::Result<(), SqliteDatabaseError> {
        // It does not matter if update is done even if moderation would be on
        // going.

        let request_info =
            serde_json::to_string(&new_request).into_error(SqliteDatabaseError::SerdeSerialize)?;
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
        .execute(self.pool())
        .await
        .into_error(SqliteDatabaseError::Execute)?;

        Ok(())
    }
}
