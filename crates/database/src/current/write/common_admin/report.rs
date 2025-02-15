use crate::{
    define_current_write_commands, DieselDatabaseError, IntoDatabaseError,
};
use diesel::{prelude::*, update, ExpressionMethods};
use error_stack::Result;
use model::{ReportProcessingState, ReportTypeNumber, UnixTime};
use model::AccountIdInternal;

define_current_write_commands!(CurrentWriteCommonAdminReport);

impl CurrentWriteCommonAdminReport<'_> {
    pub fn mark_report_done(
        &mut self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        report_type: ReportTypeNumber,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_report::dsl::*;

        let time = UnixTime::current_time();

        update(common_report)
            .filter(creator_account_id.eq(creator.as_db_id()))
            .filter(target_account_id.eq(target.as_db_id()))
            .filter(report_type_number.eq(report_type))
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
