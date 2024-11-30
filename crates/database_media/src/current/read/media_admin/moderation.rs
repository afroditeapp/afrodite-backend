use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model_media::{
    AccountId, AccountIdInternal, MediaModerationRaw, MediaModerationRequestRaw, Moderation,
    ModerationId, ModerationQueueType, ModerationRequestContent, ModerationRequestId,
    ModerationRequestState, NextQueueNumberType,
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadMediaAdminModeration);

impl CurrentReadMediaAdminModeration<'_> {
    pub fn get_in_progress_moderations(
        &mut self,
        moderator_id: AccountIdInternal,
        queue: ModerationQueueType,
    ) -> Result<Vec<Moderation>, DieselDatabaseError> {
        let data: Vec<(MediaModerationRequestRaw, AccountId)> = {
            use crate::schema::{account_id, media_moderation, media_moderation_request};

            let queue_type: NextQueueNumberType = queue.into();
            media_moderation::table
                .inner_join(media_moderation_request::table)
                .inner_join(
                    account_id::table.on(media_moderation_request::account_id.eq(account_id::id)),
                )
                .filter(media_moderation::account_id.eq(moderator_id.as_db_id()))
                .filter(media_moderation::state_number.eq(ModerationRequestState::InProgress))
                .filter(media_moderation_request::queue_number_type.eq(queue_type))
                .order(media_moderation_request::queue_number.asc())
                .select((MediaModerationRequestRaw::as_select(), account_id::uuid))
                .load(self.conn())
                .into_db_error(moderator_id)?
        };

        let new_data = data
            .into_iter()
            .map(|(moderation_request, account)| Moderation {
                request_creator_id: account,
                moderator_id: moderator_id.as_id(),
                request_id: ModerationRequestId {
                    request_row_id: moderation_request.id,
                },
                content: moderation_request.to_moderation_request_content(),
            })
            .collect();

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
