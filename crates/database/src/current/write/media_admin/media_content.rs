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
    fn update_current_security_image(
        &mut self,
        moderation_request_owner: AccountIdInternal,
        image: ContentId,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::{current_account_media::dsl::*, media_content};

        let content_id = media_content::table
            .filter(media_content::uuid.eq(image))
            .select(media_content::id)
            .first::<ContentIdDb>(self.conn())
            .into_db_error((moderation_request_owner, image))?;

        update(current_account_media.find(moderation_request_owner.as_db_id()))
            .set((security_content_id.eq(content_id),))
            .execute(self.conn())
            .into_db_error((moderation_request_owner, image))?;

        Ok(())
    }

    pub fn update_content_state(
        &mut self,
        content_id: ContentId,
        new_state: ContentState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        update(media_content.filter(uuid.eq(content_id)))
            .set((content_state.eq(new_state),))
            .execute(self.conn())
            .into_db_error((content_id, new_state))?;

        Ok(())
    }
}
