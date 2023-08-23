

use model::{
    AccountIdInternal, Moderation, ModerationId,
    ModerationRequestContent, ModerationRequestId, ModerationRequestState, ModerationRequestRaw, MediaModerationRaw,
};
use utils::IntoReportExt;
use error_stack::Result;

use diesel::prelude::*;

use crate::{IntoDatabaseError, diesel::DieselDatabaseError};




define_read_commands!(CurrentReadMediaAdmin, CurrentSyncReadMediaAdmin);




impl <'a> CurrentSyncReadMediaAdmin<'a> {
    pub fn get_in_progress_moderations(
        &'a mut self,
        moderator_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DieselDatabaseError> {
        let _account_row_id = moderator_id.row_id();
        let state_in_progress = ModerationRequestState::InProgress as i64;
        let data: Vec<(MediaModerationRaw, ModerationRequestRaw, AccountIdInternal)> = {
            use crate::schema::media_moderation::dsl::*;
            use crate::schema::media_moderation;
            use crate::schema::media_moderation_request;
            use crate::schema::account_id;

            media_moderation::table
                .inner_join(media_moderation_request::table.inner_join(account_id::table))
                .filter(account_id.eq(moderator_id.as_db_id()))
                .filter(state_number.eq(state_in_progress))
                .select((MediaModerationRaw::as_select(), ModerationRequestRaw::as_select(), AccountIdInternal::as_select()))
                .load(self.conn())
                .into_db_error(DieselDatabaseError::Execute, moderator_id)?
        };

        let mut new_data = vec![];
        for (moderation, moderation_request, account) in data.into_iter() {
            let data: ModerationRequestContent = serde_json::from_str(&moderation.json_text)
                .into_error(DieselDatabaseError::SerdeDeserialize)?;

            let moderation = Moderation {
                request_creator_id: account.as_light(),
                moderator_id: moderator_id.as_light(),
                request_id: ModerationRequestId {
                    request_row_id: moderation_request.id,
                },
                content: data,
            };
            new_data.push(moderation);
        }

        Ok(new_data)
    }

    pub fn get_next_active_moderation_request(
        &'a mut self,
        sub_queue_value: i64,
        moderator_id_for_logging: AccountIdInternal,
    ) -> Result<Option<ModerationRequestId>, DieselDatabaseError> {

        let data: Option<ModerationRequestRaw> = {
            use crate::schema::media_moderation_queue_number::dsl::*;
            use crate::schema::media_moderation_queue_number;
            use crate::schema::media_moderation_request;

            media_moderation_queue_number::table
                .inner_join(
                    media_moderation_request::table.on(
                        queue_number.eq(media_moderation_request::queue_number)
                    )
                )
                .filter(sub_queue.eq(sub_queue_value))
                .select(ModerationRequestRaw::as_select())
                .order_by(queue_number.asc())
                .first(self.conn())
                .optional()
                .into_db_error(DieselDatabaseError::Execute, moderator_id_for_logging)?
        };

        let request_row_id = match data.map(|r| r.id) {
            None => return Ok(None),
            Some(id) => id,
        };

        Ok(Some(ModerationRequestId { request_row_id }))
    }

    pub fn moderation(
        &'a mut self,
        moderation: ModerationId,
    ) -> Result<ModerationRequestContent, DieselDatabaseError> {
        let request: MediaModerationRaw = {
            use crate::schema::media_moderation::dsl::*;

            media_moderation
                .filter(account_id.eq(moderation.account_id.as_db_id()))
                .filter(moderation_request_id.eq(moderation.request_id.request_row_id))
                .select(MediaModerationRaw::as_select())
                .first(self.conn())
                .into_db_error(DieselDatabaseError::Execute, moderation)?
        };

        let data: ModerationRequestContent =
            serde_json::from_str(&request.json_text)
                .into_error(DieselDatabaseError::SerdeDeserialize)?;

        Ok(data)
    }
}
