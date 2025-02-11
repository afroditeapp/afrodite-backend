use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::ContentId;
use model_media::{AccountIdInternal, MediaStateRaw, ProfileContentEditedTime};

use crate::IntoDatabaseError;

mod media_content;
mod report;

define_current_write_commands!(CurrentWriteMedia);

pub struct DeletedSomething;

impl<'a> CurrentWriteMedia<'a> {
    pub fn media_content(self) -> media_content::CurrentWriteMediaContent<'a> {
        media_content::CurrentWriteMediaContent::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentWriteMediaReport<'a> {
        report::CurrentWriteMediaReport::new(self.cmds)
    }
}

impl CurrentWriteMedia<'_> {
    pub fn insert_media_state(&mut self, id: AccountIdInternal) -> Result<(), DieselDatabaseError> {
        use model::schema::media_state::dsl::*;

        let edit_time = ProfileContentEditedTime::current_time();

        insert_into(media_state)
            .values((
                account_id.eq(id.as_db_id()),
                initial_moderation_request_accepted.eq(false),
                profile_content_edited_unix_time.eq(edit_time),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn update_media_state(
        &mut self,
        id: AccountIdInternal,
        new: MediaStateRaw,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_state::dsl::*;

        update(media_state.find(id.as_db_id()))
            .set((initial_moderation_request_accepted.eq(new.initial_moderation_request_accepted),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn reset_media_content_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_state::dsl::*;

        update(media_state.find(id.as_db_id()))
            .set(media_content_sync_version.eq(0))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn get_next_unique_content_id(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ContentId, DieselDatabaseError> {
        use model::schema::used_content_ids::dsl::*;

        let random_cid = ContentId::new_random();

        insert_into(used_content_ids)
            .values((
                account_id.eq(id.as_db_id()),
                uuid.eq(random_cid),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(random_cid)
    }
}
