use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, ModerationQueueType, ModerationRequestId, NextQueueNumberType};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_read_commands!(
    CurrentReadMediaAdminModerationRequest,
    CurrentSyncReadMediaAdminModerationRequest
);

impl<C: ConnectionProvider> CurrentSyncReadMediaAdminModerationRequest<C> {

    /// Get the next active moderation request from the initial moderation
    /// request queue or the moderation request queue.
    pub fn get_next_active_moderation_request(
        &mut self,
        with_queue_type: ModerationQueueType,
        moderator_id_for_logging: AccountIdInternal,
    ) -> Result<Option<ModerationRequestId>, DieselDatabaseError> {
        use crate::schema::{media_moderation_request, queue_entry};

        let queue_type: NextQueueNumberType = with_queue_type.into();
        let data = queue_entry::table
            .inner_join(
                media_moderation_request::table.on(queue_entry::queue_number
                    .eq(media_moderation_request::queue_number)
                    .and(queue_entry::account_id.eq(media_moderation_request::account_id))),
            )
            .select(media_moderation_request::id)
            .order_by(media_moderation_request::queue_number.asc())
            .filter(queue_entry::queue_type_number.eq(queue_type))
            .first(self.conn())
            .optional()
            .into_db_error((moderator_id_for_logging, with_queue_type))?
            .map(|request_row_id| ModerationRequestId { request_row_id });

        Ok(data)
    }
}
