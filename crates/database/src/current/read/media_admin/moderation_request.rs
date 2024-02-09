use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, MediaModerationRequestRaw, ModerationRequestId};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_read_commands!(
    CurrentReadMediaAdminModerationRequest,
    CurrentSyncReadMediaAdminModerationRequest
);

impl<C: ConnectionProvider> CurrentSyncReadMediaAdminModerationRequest<C> {
    pub fn get_next_active_moderation_request(
        &mut self,
        _initial_moderation: bool,
        moderator_id_for_logging: AccountIdInternal,
    ) -> Result<Option<ModerationRequestId>, DieselDatabaseError> {
        let data: Option<MediaModerationRequestRaw> = {
            use crate::schema::{media_moderation_request, queue_entry};

            let first = queue_entry::table
                .inner_join(
                    media_moderation_request::table.on(queue_entry::queue_number
                        .eq(media_moderation_request::queue_number)
                        .and(queue_entry::account_id.eq(media_moderation_request::account_id))),
                )
                .select(MediaModerationRequestRaw::as_select())
                .order_by(media_moderation_request::queue_number.asc());

            // TODO
            //if initial_moderation {
            first
                .filter(media_moderation_request::account_id.is_not_null())
                .first(self.conn())
                .optional()
                .into_db_error(moderator_id_for_logging)?
            // } else {
            //     first
            //         .filter(media_moderation_request::initial_moderation_security_image.is_null())
            //         .first(self.conn())
            //         .optional()
            //         .into_db_error(moderator_id_for_logging)?
            // }
        };

        let request_row_id = match data.map(|r| r.id) {
            None => return Ok(None),
            Some(id) => id,
        };

        Ok(Some(ModerationRequestId { request_row_id }))
    }
}
