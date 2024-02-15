use diesel::{prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, ContentId, ContentIdDb, ContentState};
use simple_backend_database::diesel_db::DieselDatabaseError;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_write_commands!(
    CurrentWriteMediaAdminMediaContent,
    CurrentSyncWriteMediaAdminMediaContent
);

impl<C: ConnectionProvider> CurrentSyncWriteMediaAdminMediaContent<C> {
    pub fn update_content_state(
        &mut self,
        content_id: ContentId,
        new_state: ContentState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        update(media_content.filter(uuid.eq(content_id)))
            .set((content_state.eq(new_state)))
            .execute(self.conn())
            .into_db_error((content_id, new_state))?;

        Ok(())
    }
}
