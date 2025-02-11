use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model_media::{AccountIdInternal, MediaStateRaw};

use crate::IntoDatabaseError;

mod media_content;
mod report;

define_current_read_commands!(CurrentReadMedia);

impl<'a> CurrentReadMedia<'a> {
    pub fn media_content(self) -> media_content::CurrentReadMediaMediaContent<'a> {
        media_content::CurrentReadMediaMediaContent::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentReadMediaReport<'a> {
        report::CurrentReadMediaReport::new(self.cmds)
    }
}

impl CurrentReadMedia<'_> {
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
