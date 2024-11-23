use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model_media::{AccountIdInternal, MediaStateRaw};

use crate::IntoDatabaseError;

mod media_content;
mod moderation_request;

define_current_read_commands!(CurrentReadMedia, CurrentSyncReadMedia);

impl<C: ConnectionProvider> CurrentSyncReadMedia<C> {
    pub fn media_content(self) -> media_content::CurrentSyncReadMediaMediaContent<C> {
        media_content::CurrentSyncReadMediaMediaContent::new(self.cmds)
    }

    pub fn moderation_request(
        self,
    ) -> moderation_request::CurrentSyncReadMediaModerationRequest<C> {
        moderation_request::CurrentSyncReadMediaModerationRequest::new(self.cmds)
    }

    pub fn get_media_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<MediaStateRaw, DieselDatabaseError> {
        use crate::schema::media_state::dsl::*;

        media_state
            .filter(account_id.eq(id.as_db_id()))
            .select(MediaStateRaw::as_select())
            .first(self.conn())
            .into_db_error(id)
    }
}
