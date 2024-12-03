use database::{current::read::GetDbReadCommandsCommon, define_current_write_commands, DieselDatabaseError};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::AccountIdInternal;
use model_media::{ContentId, ContentModerationState, ProfileContentModerationRejectedReasonCategory, ProfileContentModerationRejectedReasonDetails};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteMediaAdminMediaContent);

impl CurrentWriteMediaAdminMediaContent<'_> {
    pub fn update_content_moderation_state(
        &mut self,
        content_id: ContentId,
        new_state: ContentModerationState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_content::dsl::*;

        update(media_content.filter(uuid.eq(content_id)))
            .set(moderation_state.eq(new_state))
            .execute(self.conn())
            .into_db_error(content_id)?;

        Ok(())
    }

    pub fn moderate_profile_content(
        &mut self,
        moderator_id: AccountIdInternal,
        content_id: ContentId,
        accepted: bool,
        rejected_category: Option<ProfileContentModerationRejectedReasonCategory>,
        rejected_details: Option<ProfileContentModerationRejectedReasonDetails>,
    ) -> Result<ContentModerationState, DieselDatabaseError> {
        use model::schema::media_content;

        let moderator_is_bot = self
            .read()
            .common()
            .state()
            .other_shared_state(moderator_id)?
            .is_bot_account;

        let next_state = if accepted {
            if moderator_is_bot {
                ContentModerationState::AcceptedByBot
            } else {
                ContentModerationState::AcceptedByHuman
            }
        } else if moderator_is_bot {
            ContentModerationState::RejectedByBot
        } else {
            ContentModerationState::RejectedByHuman
        };

        update(media_content::table)
            .filter(media_content::uuid.eq(content_id))
            .set((
                media_content::moderation_state.eq(next_state),
                media_content::moderation_rejected_reason_category
                    .eq(rejected_category),
                media_content::moderation_rejected_reason_details.eq(rejected_details),
                media_content::moderation_moderator_account_id
                    .eq(moderator_id.as_db_id()),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(next_state)
    }

    pub fn move_to_human_moderation(
        &mut self,
        content_id: ContentId,
    ) -> Result<ContentModerationState, DieselDatabaseError> {
        use model::schema::media_content;

        let next_state = ContentModerationState::WaitingHumanModeration;

        update(media_content::table)
            .filter(media_content::uuid.eq(content_id))
            .set(media_content::moderation_state.eq(next_state))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(next_state)
    }
}
