use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{prelude::*, update};
use error_stack::Result;
use model_media::{ContentId, ContentState};

use crate::IntoDatabaseError;

define_current_write_commands!(
    CurrentWriteMediaAdminMediaContent
);

impl CurrentWriteMediaAdminMediaContent<'_> {
    pub fn update_content_state(
        &mut self,
        content_id: ContentId,
        new_state: ContentState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        update(media_content.filter(uuid.eq(content_id)))
            .set(content_state.eq(new_state))
            .execute(self.conn())
            .into_db_error((content_id, new_state))?;

        Ok(())
    }
}
