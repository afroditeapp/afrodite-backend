use database::{
    define_current_write_commands, DieselDatabaseError,
};
use diesel::{prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model::{ReportProcessingState, UnixTime};
use model_media::AccountIdInternal;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteMediaAdminReport);

impl CurrentWriteMediaAdminReport<'_> {
    pub fn mark_report_done(
        &mut self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_report::dsl::*;

        let time = UnixTime::current_time();

        update(media_report)
            .filter(creator_account_id.eq(creator.as_db_id()))
            .filter(target_account_id.eq(target.as_db_id()))
            .set((
                moderator_account_id.eq(moderator_id.as_db_id()),
                processing_state.eq(ReportProcessingState::Done),
                processing_state_change_unix_time.eq(time),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
