use diesel::{ExpressionMethods, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, ReportIdDb, ReportProcessingState, UnixTime};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonAdminReport);

impl CurrentWriteCommonAdminReport<'_> {
    pub fn mark_report_done(
        &mut self,
        moderator_id: AccountIdInternal,
        report_id: ReportIdDb,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_report::dsl::*;

        let time = UnixTime::current_time();

        update(common_report)
            .filter(id.eq(report_id))
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
