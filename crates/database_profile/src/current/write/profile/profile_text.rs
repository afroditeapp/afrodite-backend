use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdDb, AccountIdInternal, ProfileTextModerationState};

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileText, CurrentSyncWriteProfileText);

impl<C: ConnectionProvider> CurrentSyncWriteProfileText<C> {
    pub fn reset_profile_text_moderation_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ProfileTextModerationState, DieselDatabaseError> {
        use model::schema::profile_state;

        let new_state = ProfileTextModerationState::WaitingBotOrHumanModeration;

        update(profile_state::table)
            .filter(profile_state::account_id.eq(id.as_db_id()))
            .set((
                profile_state::profile_text_moderation_state.eq(new_state),
                profile_state::profile_text_moderation_rejected_reason_category.eq(None::<i64>),
                profile_state::profile_text_moderation_rejected_reason_details.eq(None::<String>),
                profile_state::profile_text_moderation_moderator_account_id.eq(None::<AccountIdDb>),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(new_state)
    }
}
