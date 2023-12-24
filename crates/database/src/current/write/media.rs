use diesel::{delete, insert_into, prelude::*, update};
use error_stack::{Result, ResultExt, report};
use model::{
    AccountIdInternal, ContentId, ContentIdDb, ContentState, ImageSlot, ModerationQueueNumber,
    ModerationRequestContent, PrimaryImage, NextQueueNumbersRaw, ModerationRequestState, NextQueueNumberType,
};
use simple_backend_database::diesel_db::{DieselConnection, DieselDatabaseError};
use simple_backend_utils::ContextExt;

use super::ConnectionProvider;
use crate::{IntoDatabaseError, TransactionError};

define_write_commands!(CurrentWriteMedia, CurrentSyncWriteMedia);

pub struct DeletedSomething;

impl<C: ConnectionProvider> CurrentSyncWriteMedia<C> {
    pub fn insert_current_account_media(
        &mut self,
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
        &mut self,
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
        use model::schema::{current_account_media::dsl::*, media_content};

        let content_id = if let Some(content_uuid) = primary_image.content_id {
            media_content::table
                .filter(media_content::uuid.eq(content_uuid))
                .select(media_content::id)
                .first::<ContentIdDb>(conn)
                .into_db_error(DieselDatabaseError::Execute, primary_image)?
                .into()
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
        &mut self,
        content_uploader: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        // TODO: API support for secure capture

        insert_into(media_content)
            .values((
                account_id.eq(content_uploader.as_db_id()),
                uuid.eq(content_id),
                moderation_state.eq(ContentState::InSlot as i64),
                slot_number.eq(slot as i64),
                secure_capture.eq(true),
            ))
            .execute(self.conn())
            .into_db_error(
                DieselDatabaseError::Execute,
                (content_uploader, content_id, slot),
            )?;

        Ok(())
    }

    pub fn delete_image_from_slot(
        &mut self,
        request_creator: AccountIdInternal,
        slot: ImageSlot,
    ) -> Result<Option<DeletedSomething>, DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        let deleted_count = delete(
            media_content
                .filter(account_id.eq(request_creator.as_db_id()))
                .filter(moderation_state.eq(ContentState::InSlot as i64))
                .filter(slot_number.eq(slot as i64)),
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
        &mut self,
        request_creator: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        // Delete old queue number and request
        {
            use model::schema::queue_entry::dsl::*;
            delete(queue_entry.filter(
                account_id.eq(request_creator.row_id())
                    .and(queue_type_number.eq(NextQueueNumberType::MediaModeration)
                        .or(queue_type_number.eq(NextQueueNumberType::InitialMediaModeration))
                    )))
                .execute(self.conn())
                .into_db_error(DieselDatabaseError::Execute, request_creator)?;
        }
        {
            use model::schema::media_moderation_request::dsl::*;
            delete(media_moderation_request.filter(account_id.eq(request_creator.row_id())))
                .execute(self.conn())
                .into_db_error(DieselDatabaseError::Execute, request_creator)?;
        }
        // Foreign key constraint removes MediaModeration rows.
        // Old data is not needed in current data database.
        Ok(())
    }

    /// Used when a user creates a new moderation request
    ///
    /// Moderation request content must content ids which point to your own
    /// image slots. Otherwise this returns an error.
    pub fn create_new_moderation_request(
        &mut self,
        request_creator: AccountIdInternal,
        request: ModerationRequestContent,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::media_moderation_request::dsl::*;

        self.read()
            .media()
            .content_validate_moderation_request_content(request_creator, &request)?;

        // Delete old queue number and request
        self.delete_moderation_request(request_creator)?;

        let _account_row_id = request_creator.row_id();
        let queue_number_new =
            self.cmds().common().create_new_queue_entry(request_creator, NextQueueNumberType::InitialMediaModeration)?;
        let request_info = serde_json::to_string(&request)
            .change_context(DieselDatabaseError::SerdeSerialize)?;
        insert_into(media_moderation_request)
            .values((
                account_id.eq(request_creator.as_db_id()),
                queue_number.eq(queue_number_new),
            ))
            .execute(self.conn())
            .into_transaction_error(DieselDatabaseError::Execute, (request_creator, request))?;

        Ok(())
    }

    pub fn update_moderation_request(
        &mut self,
        request_owner_account_id: AccountIdInternal,
        new_request: ModerationRequestContent,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::media_moderation_request::dsl::*;

        let current_request = self.read().media().moderation_request(request_owner_account_id)?;
        let update_possible = if let Some(request) = current_request {
            request.state == ModerationRequestState::Waiting
        } else {
            false
        };

        if update_possible {
            update(media_moderation_request.find(request_owner_account_id.as_db_id()))
                .set((
                    initial_moderation_security_image.eq(new_request.initial_moderation_security_image),
                    content_id_1.eq(new_request.content1),
                    content_id_2.eq(new_request.content2),
                    content_id_3.eq(new_request.content3),
                    content_id_4.eq(new_request.content4),
                    content_id_5.eq(new_request.content5),
                    content_id_6.eq(new_request.content6),
                ))
                .execute(self.conn())
                .change_context(DieselDatabaseError::Execute)?;
            Ok(())
        } else {
            Err(DieselDatabaseError::NotAllowed.report())
        }
    }
}
