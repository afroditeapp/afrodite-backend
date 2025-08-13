use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::ContentId;
use model_media::{AccountIdInternal, ProfileContentModificationMetadata};

use crate::IntoDatabaseError;

mod media_content;
mod notification;
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
    pub fn notification(self) -> notification::CurrentWriteMediaNotification<'a> {
        notification::CurrentWriteMediaNotification::new(self.cmds)
    }
}

impl CurrentWriteMedia<'_> {
    pub fn insert_media_state(
        &mut self,
        id: AccountIdInternal,
        modification: &ProfileContentModificationMetadata,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_state::dsl::*;

        insert_into(media_state)
            .values((
                account_id.eq(id.as_db_id()),
                profile_content_edited_unix_time.eq(modification.time),
            ))
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
            .values((account_id.eq(id.as_db_id()), uuid.eq(random_cid)))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(random_cid)
    }
}
