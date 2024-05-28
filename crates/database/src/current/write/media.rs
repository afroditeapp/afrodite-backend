use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, MediaStateRaw};
use simple_backend_database::diesel_db::DieselDatabaseError;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

mod media_content;
mod moderation_request;

define_write_commands!(CurrentWriteMedia, CurrentSyncWriteMedia);

pub struct DeletedSomething;

impl<C: ConnectionProvider> CurrentSyncWriteMedia<C> {
    pub fn media_content(self) -> media_content::CurrentSyncWriteMediaContent<C> {
        media_content::CurrentSyncWriteMediaContent::new(self.cmds)
    }

    pub fn moderation_request(
        self,
    ) -> moderation_request::CurrentSyncWriteMediaModerationRequest<C> {
        moderation_request::CurrentSyncWriteMediaModerationRequest::new(self.cmds)
    }

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
