use diesel::{prelude::*, update};
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, ContentState, HandleModerationRequest, Moderation, ModerationId,
    ModerationRequestId, ModerationRequestState, NextQueueNumberType,
};
use simple_backend_database::diesel_db::DieselDatabaseError;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_write_commands!(
    CurrentWriteMediaAdminModeration,
    CurrentSyncWriteMediaAdminModeration
);

impl<C: ConnectionProvider> CurrentSyncWriteMediaAdminModeration<C> {
    pub fn moderation_get_list_and_create_new_if_necessary(
        &mut self,
        moderator_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DieselDatabaseError> {
        let mut moderations = self
            .read()
            .media_admin()
            .moderation()
            .get_in_progress_moderations(moderator_id)?;

        const MAX_COUNT: usize = 5;
        if moderations.len() >= MAX_COUNT {
            return Ok(moderations);
        }

        for _ in moderations.len()..MAX_COUNT {
            match self
                .cmds()
                .media_admin()
                .moderation()
                .create_moderation_from_next_request_in_queue(moderator_id)?
            {
                None => break,
                Some(moderation) => moderations.push(moderation),
            }
        }

        Ok(moderations)
    }

    fn create_moderation_from_next_request_in_queue(
        &mut self,
        moderator_id: AccountIdInternal,
    ) -> Result<Option<Moderation>, DieselDatabaseError> {
        let id = self
            .read()
            .media_admin()
            .moderation_request()
            .get_next_active_moderation_request(true, moderator_id)?;

        match id {
            None => Ok(None),
            Some(id) => {
                let moderation = self.create_moderation(id, moderator_id)?;
                Ok(Some(moderation))
            }
        }
    }

    fn create_moderation(
        &mut self,
        target_id: ModerationRequestId,
        moderator_id: AccountIdInternal,
    ) -> Result<Moderation, DieselDatabaseError> {
        // TODO: Currently is possible that two moderators moderate the same
        // request. Should that be prevented?

        let (request_raw, queue_number, request_creator_id) = self
            .read()
            .media()
            .moderation_request()
            .get_moderation_request_content(target_id)?;
        let content = request_raw.to_moderation_request_content();
        let _content_string =
            serde_json::to_string(&content).change_context(DieselDatabaseError::SerdeSerialize)?;

        {
            use model::schema::media_moderation::dsl::*;
            diesel::insert_into(media_moderation)
                .values((
                    moderation_request_id.eq(target_id.request_row_id),
                    account_id.eq(moderator_id.as_db_id()),
                    state_number.eq(ModerationRequestState::InProgress as i64),
                ))
                .execute(self.cmds.conn())
                .into_db_error((target_id, moderator_id))?;
        }

        // TODO
        let queue_type = if true {
            //if request_raw.initial_moderation_security_image.is_some() {
            NextQueueNumberType::InitialMediaModeration
        } else {
            NextQueueNumberType::MediaModeration
        };
        self.cmds()
            .common()
            .queue_number()
            .delete_queue_entry(queue_number.0, queue_type)?;

        let moderation = Moderation {
            request_creator_id,
            request_id: ModerationRequestId {
                request_row_id: target_id.request_row_id,
            },
            moderator_id: moderator_id.as_id(),
            content,
        };

        Ok(moderation)
    }

    /// Update moderation state of Moderation.
    ///
    /// Also updates content state.
    pub fn update_moderation(
        &mut self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DieselDatabaseError> {
        let request = self
            .read()
            .media()
            .moderation_request()
            .moderation_request(moderation_request_owner)?
            .ok_or(DieselDatabaseError::MissingModerationRequest)?;

        let _currently_selected_images = self
            .read()
            .media()
            .media_content()
            .current_account_media(moderation_request_owner)?;

        let moderation_id = ModerationId {
            request_id: ModerationRequestId {
                request_row_id: request.moderation_request_id,
            },
            account_id: moderator_id,
        };

        let content = self
            .read()
            .media_admin()
            .moderation()
            .moderation(moderation_id)?;

        let state = if result.accept {
            ModerationRequestState::Accepted
        } else {
            ModerationRequestState::Denied
        };

        let new_content_state = match state {
            ModerationRequestState::Accepted => ContentState::ModeratedAsAccepted,
            ModerationRequestState::Denied => ContentState::ModeratedAsDenied,
            ModerationRequestState::InProgress => ContentState::InModeration,
            ModerationRequestState::Waiting => ContentState::InSlot,
        };

        for c in content.content() {
            let _content_info = self
                .read()
                .media()
                .media_content()
                .get_media_content_raw(c)?;
            // TODO
            //let is_security = if let Some(content) = content.initial_moderation_security_image {
            let _is_security = if true {
                //content == c
                true
            } else {
                false
            };
            self.cmds()
                .media_admin()
                .media_content()
                .update_content_state(
                    c,
                    new_content_state,
                    // is_security,
                )?;
        }

        // TODO
        // if let Some(security_image) = content.initial_moderation_security_image {
        //     if state == ModerationRequestState::Accepted
        //     && currently_selected_images.security_content_id.is_none() {
        //         self.update_current_security_image(
        //             moderation_request_owner,
        //             security_image,
        //         )?;

        //         let primary_image = PrimaryImage {
        //             //content_id: Some(content.content1),
        //             content_id: Some(content.content0),
        //             grid_crop_size: 0.0,
        //             grid_crop_x: 0.0,
        //             grid_crop_y: 0.0,
        //         };

        //         self.cmds().media().update_current_account_media_with_primary_image(
        //             moderation_request_owner,
        //             primary_image,
        //         )?;
        //     }

        // }

        let _state_number = state as i64;

        {
            use model::schema::media_moderation::dsl::*;
            update(media_moderation)
                .filter(account_id.eq(moderation_id.account_id.as_db_id()))
                .filter(moderation_request_id.eq(moderation_id.request_id.request_row_id))
                .set(state_number.eq(state as i64))
                .execute(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }
}
