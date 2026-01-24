use database::{
    DieselDatabaseError, IntoDatabaseError, current::read::GetDbReadCommandsCommon,
    define_current_read_commands,
};
use diesel::prelude::*;
use error_stack::Result;
use model_profile::{
    AccountIdInternal, GetProfileStringPendingModerationList,
    GetProfileStringPendingModerationParams, ProfileStringModerationContentType,
    ProfileStringModerationState, ProfileStringPendingModeration,
};

define_current_read_commands!(CurrentReadProfileModeration);

impl CurrentReadProfileModeration<'_> {
    pub fn profile_string_pending_moderation_list_using_moderator_id(
        &mut self,
        moderator_id: AccountIdInternal,
        params: GetProfileStringPendingModerationParams,
    ) -> Result<GetProfileStringPendingModerationList, DieselDatabaseError> {
        let is_bot = self
            .read()
            .common()
            .state()
            .other_shared_state(moderator_id)?
            .is_bot_account;
        self.profile_string_pending_moderation_list(is_bot, params)
    }

    pub fn profile_string_pending_moderation_list(
        &mut self,
        is_bot: bool,
        params: GetProfileStringPendingModerationParams,
    ) -> Result<GetProfileStringPendingModerationList, DieselDatabaseError> {
        use crate::schema::{account_id, profile, profile_moderation};

        const LIMIT: i64 = 25;

        let is_bot =
            diesel::expression::AsExpression::<diesel::sql_types::Bool>::as_expression(is_bot);
        let is_not_bot = is_bot.eq(false);

        let show_bot_moderations =
            diesel::expression::AsExpression::<diesel::sql_types::Bool>::as_expression(
                params.show_values_which_bots_can_moderate,
            );

        let query = profile::table
            .inner_join(account_id::table)
            .inner_join(
                profile_moderation::table.on(profile_moderation::account_id.eq(account_id::id)),
            )
            .filter(profile_moderation::content_type.eq(params.content_type))
            .filter(
                show_bot_moderations
                    .and(
                        profile_moderation::state_type
                            .eq(ProfileStringModerationState::WaitingBotOrHumanModeration),
                    )
                    .or(is_not_bot.and(
                        profile_moderation::state_type
                            .eq(ProfileStringModerationState::WaitingHumanModeration),
                    )),
            )
            .order((
                profile_moderation::created_unix_time.asc(),
                account_id::id.asc(),
            ))
            .limit(LIMIT);

        let values = match params.content_type {
            ProfileStringModerationContentType::ProfileName => query
                .filter(profile::profile_name.is_not_null())
                .select((
                    account_id::uuid,
                    profile::profile_name.assume_not_null(),
                    profile_moderation::rejected_reason_category,
                    profile_moderation::rejected_reason_details,
                ))
                .load::<ProfileStringPendingModeration>(self.conn()),
            ProfileStringModerationContentType::ProfileText => query
                .filter(profile::profile_text.is_not_null())
                .select((
                    account_id::uuid,
                    profile::profile_text.assume_not_null(),
                    profile_moderation::rejected_reason_category,
                    profile_moderation::rejected_reason_details,
                ))
                .load::<ProfileStringPendingModeration>(self.conn()),
        }
        .into_db_error(())?;

        Ok(GetProfileStringPendingModerationList { values })
    }
}
