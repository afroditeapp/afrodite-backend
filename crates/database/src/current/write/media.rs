use model::{
    AccountIdInternal, ContentId, ContentState, CurrentAccountMediaInternal, ImageSlot,
    ModerationRequest, ModerationRequestContent, PrimaryImage, QueueNumberRaw, ModerationQueueNumber, ContentIdInternal, ContentIdDb,
};
use sqlx::{Sqlite, Transaction};
use utils::IntoReportExt;
use diesel::{prelude::*, update, insert_into, delete};
use error_stack::Result;

use crate::{sqlite::SqliteDatabaseError, WriteResult, diesel::{DieselDatabaseError, DieselConnection}, IntoDatabaseError, TransactionError};

define_write_commands!(CurrentWriteMedia, CurrentSyncWriteMedia);

pub struct DeletedSomething;

impl<'a> CurrentSyncWriteMedia<'a> {

    pub fn insert_current_account_media(
        &'a mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        insert_into(current_account_media)
            .values(account_id.eq(id.as_db_id()))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn primary_image(
        &'a mut self,
        id: AccountIdInternal,
        primary_image: PrimaryImage,
    ) -> Result<(), DieselDatabaseError> {
        Self::update_current_account_media_with_primary_image(self.conn(), id, primary_image)
    }

    pub fn update_current_account_media_with_primary_image(
        conn: &mut DieselConnection,
        id: AccountIdInternal,
        primary_image: PrimaryImage,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;
        use model::schema::media_content;

        let content_id = if let Some(content_uuid) = primary_image.content_id {
            media_content::table.filter(media_content::uuid.eq(content_uuid))
                .select(media_content::id)
                .first::<ContentIdDb>(conn)
                .into_db_error(DieselDatabaseError::Execute, primary_image)?.into()
        } else {
            None
        };

        update(current_account_media.find(id.as_db_id()))
            .set((
                profile_content_id.eq(content_id),
                grid_crop_size.eq(primary_image.grid_crop_size),
                grid_crop_x.eq(primary_image.grid_crop_x),
                grid_crop_y.eq(primary_image.grid_crop_y),
            ))
            .execute(conn)
            .into_db_error(DieselDatabaseError::Execute, (id, primary_image))?;

        Ok(())
    }

    pub fn insert_content_id_to_slot(
        transaction_conn: &mut DieselConnection,
        content_uploader: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        insert_into(media_content)
            .values((
                account_id.eq(content_uploader.as_db_id()),
                uuid.eq(content_id),
                moderation_state.eq(ContentState::InSlot as i64),
                slot_number.eq(slot as i64),

            ))
            .execute(transaction_conn)
            .into_db_error(DieselDatabaseError::Execute, (content_uploader, content_id, slot))?;

        Ok(())
    }

    pub fn delete_image_from_slot(
        &'a mut self,
        request_creator: AccountIdInternal,
        slot: ImageSlot,
    ) -> Result<Option<DeletedSomething>, DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        let deleted_count = delete(
            media_content
                .filter(account_id.eq(request_creator.as_db_id()))
                .filter(moderation_state.eq(ContentState::InSlot as i64))
                .filter(slot_number.eq(slot as i64))
        )
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (request_creator, slot))?;

        if deleted_count > 0 {
            Ok(Some(DeletedSomething))
        } else {
            Ok(None)
        }
    }

    fn delete_moderation_request(
        transaction_conn: &mut SqliteConnection,
        request_creator: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        // Delete old queue number and request
        {
            use model::schema::media_moderation_queue_number::dsl::*;
            delete(
                media_moderation_queue_number
                    .filter(account_id.eq(request_creator.row_id()))
            )
                .execute(transaction_conn)
                .into_db_error(DieselDatabaseError::Execute, request_creator)?;
        }
        {
            use model::schema::media_moderation_request::dsl::*;
            delete(
                media_moderation_request
                    .filter(account_id.eq(request_creator.row_id()))
            )
                .execute(transaction_conn)
                .into_db_error(DieselDatabaseError::Execute, request_creator)?;
        }
        // Foreign key constraint removes MediaModeration rows.
        // Old data is not needed in current data database.
        Ok(())
    }

    /// Used when a user creates a new moderation request
    fn create_new_moderation_request_queue_number(
        transaction_conn: &mut SqliteConnection,
        request_creator: AccountIdInternal,
    ) -> Result<ModerationQueueNumber, DieselDatabaseError> {
        use model::schema::media_moderation_queue_number::dsl::*;

        insert_into(media_moderation_queue_number)
            .values((
                account_id.eq(request_creator.as_db_id()),
                sub_queue.eq(0))  // TODO: set to correct queue, for example if premium account
            )
            .returning(QueueNumberRaw::as_returning())
            .get_result(transaction_conn)
            .map(|data| data.queue_number)
            .into_db_error(DieselDatabaseError::Execute, request_creator)
    }

    /// Used when a user creates a new moderation request
    ///
    /// Moderation request content must content ids which point to your own
    /// image slots. Otherwise this returns an error.
    pub fn create_new_moderation_request(
        &'a mut self,
        request_creator: AccountIdInternal,
        request: ModerationRequestContent,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::media_moderation_request::dsl::*;

        self.cmds.read()
            .media()
            .content_validate_moderation_request_content(request_creator, &request)?;

        self.conn().transaction(|conn| {
            // Delete old queue number and request
            Self::delete_moderation_request(conn, request_creator)?;

            let account_row_id = request_creator.row_id();
            let queue_number_new =
                Self::create_new_moderation_request_queue_number(
                    conn,
                    request_creator
                )?;
            let request_info =
                serde_json::to_string(&request).into_error(DieselDatabaseError::SerdeSerialize)?;
            insert_into(media_moderation_request)
                .values((
                    account_id.eq(request_creator.as_db_id()),
                    queue_number.eq(queue_number_new),
                    json_text.eq(request_info)
                ))
                .execute(conn)
                .into_transaction_error(DieselDatabaseError::Execute, (request_creator, request))?;

            Ok::<_, TransactionError<_>>(())
        })?;

        Ok(())
    }

    pub fn update_moderation_request(
        &'a mut self,
        request_owner_account_id: AccountIdInternal,
        new_request: ModerationRequestContent,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::media_moderation_request::dsl::*;

        // It does not matter if update is done even if moderation would be on
        // going.

        let request_info =
            serde_json::to_string(&new_request).into_error(DieselDatabaseError::SerdeSerialize)?;

        update(media_moderation_request.find(request_owner_account_id.as_db_id()))
            .set(json_text.eq(request_info))
            .execute(self.conn())
            .into_error(DieselDatabaseError::Execute)?;

        Ok(())
    }
}
