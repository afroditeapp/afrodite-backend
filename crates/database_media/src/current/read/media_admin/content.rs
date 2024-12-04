use database::{current::read::GetDbReadCommandsCommon, define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model_media::{
    AccountIdInternal, ContentModerationState, GetProfileContentPendingModerationList, GetProfileContentPendingModerationParams, ModerationQueueType, ProfileContentPendingModeration
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadMediaAdminContent);

impl CurrentReadMediaAdminContent<'_> {
    pub fn profile_content_pending_moderation_list(
        &mut self,
        moderator_id: AccountIdInternal,
        params: GetProfileContentPendingModerationParams,
    ) -> Result<GetProfileContentPendingModerationList, DieselDatabaseError> {
        use crate::schema::{account_id, media_content};

        const LIMIT: i64 = 25;

        let is_bot = self
            .read()
            .common()
            .state()
            .other_shared_state(moderator_id)?
            .is_bot_account;
        let is_bot =
            diesel::expression::AsExpression::<diesel::sql_types::Bool>::as_expression(is_bot);
        let is_not_bot = is_bot.eq(false);

        let show_bot_moderations =
            diesel::expression::AsExpression::<diesel::sql_types::Bool>::as_expression(
                params.show_content_which_bots_can_moderate,
            );

        let initial_content_value = match params.queue {
            ModerationQueueType::InitialMediaModeration => true,
            ModerationQueueType::MediaModeration => false,
        };

        let values = media_content::table
            .inner_join(account_id::table)
            .filter(
                show_bot_moderations
                    .and(
                        media_content::moderation_state
                            .eq(ContentModerationState::WaitingBotOrHumanModeration),
                    )
                    .or(is_not_bot.and(
                        media_content::moderation_state
                            .eq(ContentModerationState::WaitingHumanModeration),
                    )),
            )
            .filter(media_content::content_type_number.eq(params.content_type))
            .filter(media_content::initial_content.eq(initial_content_value))
            .select((account_id::uuid, media_content::uuid))
            .order((
                media_content::creation_unix_time.asc(),
                account_id::id.asc(),
            ))
            .limit(LIMIT)
            .load::<ProfileContentPendingModeration>(self.conn())
            .into_db_error(())?;

        Ok(GetProfileContentPendingModerationList { values })
    }
}
