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
            .into_db_error(
                DieselDatabaseError::Execute,
                (moderation_request_owner, image),
            )?;

        update(current_account_media.find(moderation_request_owner.as_db_id()))
            .set((security_content_id.eq(content_id),))
            .execute(self.conn())
            .into_db_error(
                DieselDatabaseError::Execute,
                (moderation_request_owner, image),
            )?;

        Ok(())
    }

    // fn update_current_primary_image_from_slot_2(
    //     transaction_conn: &mut DieselConnection,
    //     moderation_request_owner: AccountIdInternal,
    //     content: ModerationRequestContent,
    // ) -> Result<(), DieselDatabaseError> {
    //     use model::schema::current_account_media::dsl::*;

    //     let request_owner_id = moderation_request_owner.row_id();
    //     let primary_img_content_id = content
    //         .slot_2()
    //         .ok_or(DieselDatabaseError::ContentSlotEmpty)?
    //         .content_id;

    //     update(media_content.filter(uuid.eq(content_id)))
    //         .set((
    //             moderation_state.eq(state),
    //             content_type.eq(content_type_value),
    //         ))
    //         .execute(transaction_conn)
    //         .into_db_error(DieselDatabaseError::Execute, (content_id, new_state))?;

    //     sqlx::query!(
    //         r#"
    //         UPDATE CurrentAccountMedia
    //         SET profile_content_row_id = mc.content_row_id
    //         FROM (SELECT content_id, content_row_id FROM MediaContent) AS mc
    //         WHERE account_row_id = ? AND mc.content_id = ?
    //         "#,
    //         request_owner_id,
    //         primary_img_content_id,
    //     )
    //     .execute(&mut **transaction)
    //     .await
    //     .change_context(SqliteDatabaseError::Execute)?;

    //     Ok(())
    // }

    pub fn update_content_state(
        &mut self,
        content_id: ContentId,
        new_state: ContentState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        update(media_content.filter(uuid.eq(content_id)))
            .set((content_state.eq(new_state),))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (content_id, new_state))?;

        Ok(())
    }
}
