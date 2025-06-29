use database::{
    DieselDatabaseError, IntoDatabaseError, current::read::GetDbReadCommandsCommon,
    define_current_read_commands,
};
use diesel::prelude::*;
use error_stack::Result;
use model_profile::{
    AccountIdInternal, GetProfileTextPendingModerationList, GetProfileTextPendingModerationParams,
    ProfileTextModerationState, ProfileTextPendingModeration,
};

define_current_read_commands!(CurrentReadProfileText);

impl CurrentReadProfileText<'_> {
    pub fn profile_text_pending_moderation_list_using_moderator_id(
        &mut self,
        moderator_id: AccountIdInternal,
        params: GetProfileTextPendingModerationParams,
    ) -> Result<GetProfileTextPendingModerationList, DieselDatabaseError> {
        let is_bot = self
            .read()
            .common()
            .state()
            .other_shared_state(moderator_id)?
            .is_bot_account;
        self.profile_text_pending_moderation_list(is_bot, params)
    }

    pub fn profile_text_pending_moderation_list(
        &mut self,
        is_bot: bool,
        params: GetProfileTextPendingModerationParams,
    ) -> Result<GetProfileTextPendingModerationList, DieselDatabaseError> {
        use crate::schema::{account_id, profile, profile_state};

        const LIMIT: i64 = 25;

        let is_bot =
            diesel::expression::AsExpression::<diesel::sql_types::Bool>::as_expression(is_bot);
        let is_not_bot = is_bot.eq(false);

        let show_bot_moderations =
            diesel::expression::AsExpression::<diesel::sql_types::Bool>::as_expression(
                params.show_texts_which_bots_can_moderate,
            );

        let values = profile::table
            .inner_join(account_id::table)
            .inner_join(profile_state::table.on(profile_state::account_id.eq(account_id::id)))
            .filter(
                show_bot_moderations
                    .and(
                        profile_state::profile_text_moderation_state
                            .eq(ProfileTextModerationState::WaitingBotOrHumanModeration),
                    )
                    .or(is_not_bot.and(
                        profile_state::profile_text_moderation_state
                            .eq(ProfileTextModerationState::WaitingHumanModeration),
                    )),
            )
            .select((account_id::uuid, profile::profile_text))
            .order((
                profile_state::profile_text_edit_time_unix_time.asc(),
                account_id::id.asc(),
            ))
            .limit(LIMIT)
            .load::<ProfileTextPendingModeration>(self.conn())
            .into_db_error(())?;

        Ok(GetProfileTextPendingModerationList { values })
    }
}
