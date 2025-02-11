use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, ReportProcessingState};
use simple_backend_utils::current_unix_time;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileReport);

impl CurrentWriteProfileReport<'_> {
    pub fn upsert_profile_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        text: Option<String>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_report::dsl::*;

        let time = current_unix_time();

        let state = if text.is_some() {
            ReportProcessingState::Empty
        } else {
            ReportProcessingState::Waiting
        };

        insert_into(profile_report)
            .values((
                creator_account_id.eq(creator.as_db_id()),
                target_account_id.eq(target.as_db_id()),
                creation_unix_time.eq(time),
                content_edit_unix_time.eq(time),
                processing_state.eq(state),
                processing_state_change_unix_time.eq(time),
                profile_text.eq(&text),
            ))
            .on_conflict((creator_account_id, target_account_id))
            .do_update()
            .set((
                content_edit_unix_time.eq(time),
                processing_state.eq(state),
                processing_state_change_unix_time.eq(time),
                profile_text.eq(&text),
            ))
            .execute(self.conn())
            .into_db_error((creator, target))?;

        Ok(())
    }
}
