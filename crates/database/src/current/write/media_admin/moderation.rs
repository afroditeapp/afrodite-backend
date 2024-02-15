use diesel::{prelude::*, update};
use error_stack::{Result, ResultExt};
use model::{
    media, schema::media_state::initial_moderation_request_accepted, AccountIdInternal, ContentState, HandleModerationRequest, Moderation, ModerationId, ModerationQueueNumber, ModerationQueueType, ModerationRequestId, ModerationRequestState, NextQueueNumberType
};
use simple_backend_database::diesel_db::DieselDatabaseError;
use tokio_stream::Elapsed;

use super::{ConnectionProvider, InitialModerationRequestIsNowAccepted};
use crate::IntoDatabaseError;

define_write_commands!(
    CurrentWriteMediaAdminModeration,
    CurrentSyncWriteMediaAdminModeration
);

// TODO(prod): Support selecting initial and normal requests from API
// level, so that admin can prioritize moderations.

impl<C: ConnectionProvider> CurrentSyncWriteMediaAdminModeration<C> {
    pub fn moderation_get_list_and_create_new_if_necessary(
        &mut self,
        moderator_id: AccountIdInternal,
        queue: ModerationQueueType,
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
                .create_moderation_from_next_request_in_queue(moderator_id, queue)?
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
        queue: ModerationQueueType,
    ) -> Result<Option<Moderation>, DieselDatabaseError> {
        let id = self
            .read()
            .media_admin()
            .moderation_request()
            .get_next_active_moderation_request(queue, moderator_id)?;

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

        let (request_raw, request_creator_id) = self
            .read()
            .media()
            .moderation_request()
            .get_moderation_request_content(target_id)?;
        let content = request_raw.to_moderation_request_content();

        {
            use model::schema::media_moderation::dsl::*;
            diesel::insert_into(media_moderation)
                .values((
                    moderation_request_id.eq(target_id.request_row_id),
                    account_id.eq(moderator_id.as_db_id()),
                    state_number.eq(ModerationRequestState::InProgress),
                ))
                .execute(self.cmds.conn())
                .into_db_error((target_id, moderator_id))?;
        }

        let media_state =
            self.read().media().get_media_state(request_creator_id)?;

        let queue_type = if media_state.current_moderation_request_is_initial() {
            NextQueueNumberType::InitialMediaModeration
        } else {
            NextQueueNumberType::MediaModeration
        };
        self.cmds()
            .common()
            .queue_number()
            .delete_queue_entry(request_raw.queue_number, queue_type)?;

        let moderation = Moderation {
            request_creator_id: request_creator_id.into(),
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
    ) -> Result<Option<InitialModerationRequestIsNowAccepted>, DieselDatabaseError> {
        let request = self
            .read()
            .media()
            .moderation_request()
            .moderation_request(moderation_request_owner)?
            .ok_or(DieselDatabaseError::MissingModerationRequest)?;

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

        let new_content_state = if result.accept {
            ContentState::ModeratedAsAccepted
        } else {
            ContentState::ModeratedAsDenied
        };

        for c in content.iter() {
            let content_info = self
                .read()
                .media()
                .media_content()
                .get_media_content_raw(c)?;
            // The first moderation is the final moderation.
            // (In case the above TODO is true: are multiple
            //  moderations possible?)
            if content_info.content_state == ContentState::InModeration {
                self.cmds()
                    .media_admin()
                    .media_content()
                    .update_content_state(
                        c,
                        new_content_state,
                    )?;
            }
        }

        {
            use model::schema::media_moderation::dsl::*;
            update(media_moderation)
                .filter(account_id.eq(moderation_id.account_id.as_db_id()))
                .filter(moderation_request_id.eq(moderation_id.request_id.request_row_id))
                .set(state_number.eq(state))
                .execute(self.conn())
                .into_db_error(())?;
        }

        if new_content_state == ContentState::ModeratedAsAccepted {
            let mut media_state =
                self.read().media().get_media_state(moderation_request_owner)?;
            if media_state.current_moderation_request_is_initial() {
                media_state.initial_moderation_request_accepted = true;
                self.cmds().media().update_media_state(moderation_request_owner, media_state)?;

                // Move pending content to current content.
                self.cmds().media().media_content().move_pending_content_to_current_content(moderation_request_owner)?;

                return Ok(Some(InitialModerationRequestIsNowAccepted));
            }
        }

        Ok(None)
    }
}
