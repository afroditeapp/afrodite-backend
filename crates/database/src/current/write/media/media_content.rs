
use diesel::{insert_into, prelude::*, update};
use error_stack::{Result};
use model::{
    AccountIdInternal, ContentId, ContentState, SetProfileContent, MediaContentType, SetProfileContentInternal,
};
use simple_backend_database::diesel_db::{DieselDatabaseError};
use simple_backend_utils::ContextExt;

use super::ConnectionProvider;
use crate::{IntoDatabaseError};

define_write_commands!(CurrentWriteMediaContent, CurrentSyncWriteMediaContent);

impl<C: ConnectionProvider> CurrentSyncWriteMediaContent<C> {
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

    pub fn update_profile_content_if_possible(
        &mut self,
        id: AccountIdInternal,
        new: SetProfileContent,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        let all_content = self.read().media().media_content().get_account_media_content(id)?;
        let convert = |content_id: Option<ContentId>| {
            if let Some(content_id) = content_id {
                let found = all_content
                    .iter()
                    .find(|content|
                        content.content_id == content_id &&
                        content.state == ContentState::ModeratedAsAccepted &&
                        content.content_type == MediaContentType::JpegImage
                    );

                if let Some(c) = found {
                    Ok(Some(c.content_row_id))
                } else {
                    Err(DieselDatabaseError::NotAllowed.report())
                }
            } else {
                Ok(None)
            }
        };

        update(current_account_media.find(id.as_db_id()))
            .set((
                profile_content_id_0.eq(convert(Some(new.content_id_0))?),
                profile_content_id_1.eq(convert(new.content_id_1)?),
                profile_content_id_2.eq(convert(new.content_id_2)?),
                profile_content_id_3.eq(convert(new.content_id_3)?),
                profile_content_id_4.eq(convert(new.content_id_4)?),
                profile_content_id_5.eq(convert(new.content_id_5)?),
                grid_crop_size.eq(new.grid_crop_size),
                grid_crop_x.eq(new.grid_crop_x),
                grid_crop_y.eq(new.grid_crop_y),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (id, new))?;

        Ok(())
    }

    pub fn update_or_delete_pending_profile_content_if_possible(
        &mut self,
        id: AccountIdInternal,
        new: Option<SetProfileContent>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::current_account_media::dsl::*;

        let new = if let Some(new) = new {
            // Update
            new.into()
        } else {
            // Delete
            SetProfileContentInternal::default()
        };

        let all_content = self.read().media().media_content().get_account_media_content(id)?;
        let convert = |content_id: Option<ContentId>| {
            if let Some(content_id) = content_id {
                let found = all_content
                    .iter()
                    .find(|content|
                        content.content_id == content_id &&
                        content.state != ContentState::ModeratedAsDenied &&
                        content.content_type == MediaContentType::JpegImage
                    );

                if let Some(c) = found {
                    Ok(Some(c.content_row_id))
                } else {
                    Err(DieselDatabaseError::NotAllowed.report())
                }
            } else {
                Ok(None)
            }
        };

        update(current_account_media.find(id.as_db_id()))
            .set((
                pending_profile_content_id_0.eq(convert(new.content_id_0)?),
                pending_profile_content_id_1.eq(convert(new.content_id_1)?),
                pending_profile_content_id_2.eq(convert(new.content_id_2)?),
                pending_profile_content_id_3.eq(convert(new.content_id_3)?),
                pending_profile_content_id_4.eq(convert(new.content_id_4)?),
                pending_profile_content_id_5.eq(convert(new.content_id_5)?),
                pending_grid_crop_size.eq(new.grid_crop_size),
                pending_grid_crop_x.eq(new.grid_crop_x),
                pending_grid_crop_y.eq(new.grid_crop_y),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (id, new))?;

        Ok(())
    }


    pub fn update_security_image(
        &mut self,
        _image_owner: AccountIdInternal,
        _content_id: ContentId,
    ) -> Result<(), DieselDatabaseError> {

        unimplemented!("TODO")
    }

    pub fn update_or_delete_pending_security_image(
        &mut self,
        _image_owner: AccountIdInternal,
        _content_id: Option<ContentId>,
    ) -> Result<(), DieselDatabaseError> {

        unimplemented!("TODO")
    }

    pub fn delete_content(
        &mut self,
        _content_owner: AccountIdInternal,
        _content_id: ContentId,
    ) -> Result<(), DieselDatabaseError> {
        // // Delete old queue number and request
        // {
        //     use model::schema::queue_entry::dsl::*;
        //     delete(queue_entry.filter(
        //         account_id.eq(request_creator.row_id())
        //             .and(queue_type_number.eq(NextQueueNumberType::MediaModeration)
        //                 .or(queue_type_number.eq(NextQueueNumberType::InitialMediaModeration))
        //             )))
        //         .execute(self.conn())
        //         .into_db_error(DieselDatabaseError::Execute, request_creator)?;
        // }
        // {
        //     use model::schema::media_moderation_request::dsl::*;
        //     delete(media_moderation_request.filter(account_id.eq(request_creator.row_id())))
        //         .execute(self.conn())
        //         .into_db_error(DieselDatabaseError::Execute, request_creator)?;
        // }
        // // Foreign key constraint removes MediaModeration rows.
        // // Old data is not needed in current data database.
        // Ok(())
        unimplemented!("TODO")
    }
}
