use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{GetProfileTextPendingModerationList, GetProfileTextPendingModerationParams, ProfileTextModerationState, ProfileTextPendingModeration};
use database::IntoDatabaseError;

define_current_read_commands!(CurrentReadProfileText, CurrentSyncReadProfileText);

impl<C: ConnectionProvider> CurrentSyncReadProfileText<C> {
    pub fn profile_text_pending_moderation_list(
        &mut self,
        params: GetProfileTextPendingModerationParams,
    ) -> Result<GetProfileTextPendingModerationList, DieselDatabaseError> {
        use crate::schema::{profile, account_id, profile_state};

        const LIMIT: i64 = 25;

        // TODO(prod): Implement show_texts_which_bots_can_moderate

        let values = profile::table
            .inner_join(account_id::table)
            .inner_join(
                profile_state::table.on(profile_state::account_id.eq(account_id::id)),
            )
            .filter(
                profile_state::profile_text_moderation_state.eq(ProfileTextModerationState::WaitingBotOrHumanModeration)
                    .or(profile_state::profile_text_moderation_state.eq(ProfileTextModerationState::WaitingHumanModeration))
            )
            .select((
                account_id::uuid,
                profile::profile_text,
            ))
            .order((
                profile_state::profile_text_edit_time_unix_time.asc(),
                account_id::id.asc(),
            ))
            .limit(LIMIT)
            .load::<ProfileTextPendingModeration>(self.conn())
            .into_db_error(())?;

        Ok(GetProfileTextPendingModerationList {
            values,
        })
    }
}
