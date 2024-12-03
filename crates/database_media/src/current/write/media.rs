use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model_media::{AccountIdInternal, MediaStateRaw};

use crate::IntoDatabaseError;

mod media_content;

define_current_write_commands!(CurrentWriteMedia);

pub struct DeletedSomething;

impl<'a> CurrentWriteMedia<'a> {
    pub fn media_content(self) -> media_content::CurrentWriteMediaContent<'a> {
        media_content::CurrentWriteMediaContent::new(self.cmds)
    }
}

impl CurrentWriteMedia<'_> {
    pub fn insert_media_state(&mut self, id: AccountIdInternal) -> Result<(), DieselDatabaseError> {
        use model::schema::media_state::dsl::*;

        insert_into(media_state)
            .values((
                account_id.eq(id.as_db_id()),
                initial_moderation_request_accepted.eq(false),
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
}
