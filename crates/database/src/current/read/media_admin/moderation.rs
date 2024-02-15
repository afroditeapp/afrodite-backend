use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountIdInternal, MediaModerationRaw, MediaModerationRequestRaw, Moderation, ModerationId, ModerationQueueType, ModerationRequestContent, ModerationRequestId, ModerationRequestState, NextQueueNumberType
};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_read_commands!(
    CurrentReadMediaAdminModeration,
    CurrentSyncReadMediaAdminModeration
);

impl<C: ConnectionProvider> CurrentSyncReadMediaAdminModeration<C> {
    pub fn get_in_progress_moderations(
        &mut self,
        moderator_id: AccountIdInternal,
        queue: ModerationQueueType,
    ) -> Result<Vec<Moderation>, DieselDatabaseError> {
        let data: Vec<(
            MediaModerationRequestRaw,
            AccountIdInternal,
        )> = {
            use crate::schema::{
                account_id, media_moderation, media_moderation::dsl::*, media_moderation_request, queue_entry,
            };

            let queue_type: NextQueueNumberType = queue.into();
            media_moderation::table
                .inner_join(media_moderation_request::table)
                .inner_join(account_id::table)
                .inner_join(
                    queue_entry::table.on(
                        queue_entry::queue_number
                            .eq(media_moderation_request::queue_number)
                    )
                )
                .filter(account_id.eq(moderator_id.as_db_id()))
                .filter(state_number.eq(ModerationRequestState::InProgress))
                .filter(queue_entry::queue_type_number.eq(queue_type))
                .select((
                    MediaModerationRequestRaw::as_select(),
                    AccountIdInternal::as_select(),
                ))
                .load(self.conn())
                .into_db_error(moderator_id)?
        };

        let new_data = data.into_iter().map(|(moderation_request, account)| {
            Moderation {
                request_creator_id: account.as_id(),
                moderator_id: moderator_id.as_id(),
                request_id: ModerationRequestId {
                    request_row_id: moderation_request.id,
                },
                content: moderation_request.to_moderation_request_content(),
            }
        }).collect();

        Ok(new_data)
    }

    pub fn moderation(
        &mut self,
        moderation: ModerationId,
    ) -> Result<ModerationRequestContent, DieselDatabaseError> {
        let (_moderation, request) = {
            use crate::schema::{media_moderation, media_moderation_request};

            media_moderation::table
                .inner_join(media_moderation_request::table)
                .filter(media_moderation::account_id.eq(moderation.account_id.as_db_id()))
                .filter(
                    media_moderation::moderation_request_id
                        .eq(moderation.request_id.request_row_id),
                )
                .select((
                    MediaModerationRaw::as_select(),
                    MediaModerationRequestRaw::as_select(),
                ))
                .first(self.conn())
                .into_db_error(moderation)?
        };

        Ok(request.to_moderation_request_content())
    }
}
