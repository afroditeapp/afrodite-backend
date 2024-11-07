use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdDb, AccountIdInternal, ProfileTextModerationState, UnixTime};

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileText, CurrentSyncWriteProfileText);

impl<C: ConnectionProvider> CurrentSyncWriteProfileText<C> {
    pub fn reset_profile_text_moderation_state(
        &mut self,
        id: AccountIdInternal,
        text_empty: bool,
    ) -> Result<ProfileTextModerationState, DieselDatabaseError> {
        use model::schema::profile_state;

        let new_state = if text_empty {
            ProfileTextModerationState::Empty
        } else {
            ProfileTextModerationState::WaitingBotOrHumanModeration
        };
        let current_time = UnixTime::current_time();

        update(profile_state::table)
            .filter(profile_state::account_id.eq(id.as_db_id()))
            .set((
                profile_state::profile_text_moderation_state.eq(new_state),
                profile_state::profile_text_moderation_rejected_reason_category.eq(None::<i64>),
                profile_state::profile_text_moderation_rejected_reason_details.eq(None::<String>),
                profile_state::profile_text_moderation_moderator_account_id.eq(None::<AccountIdDb>),
                profile_state::profile_text_edit_time_unix_time.eq(current_time),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(new_state)
    }
}
