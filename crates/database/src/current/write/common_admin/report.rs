use diesel::{ExpressionMethods, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, ReportIdDb, ReportProcessingState, UnixTime};

use crate::{
    DieselDatabaseError, IntoDatabaseError, current::read::GetDbReadCommandsCommon,
    define_current_write_commands,
};

define_current_write_commands!(CurrentWriteCommonAdminReport);

impl CurrentWriteCommonAdminReport<'_> {
    pub fn mark_report_processed(
        &mut self,
        moderator_id: AccountIdInternal,
        report_id: ReportIdDb,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_report::dsl::*;

        let time = UnixTime::current_time();
        let is_bot = self
            .read()
            .common()
            .state()
            .other_shared_state(moderator_id)?
            .is_bot();
        let state = if is_bot {
            ReportProcessingState::ProcessedByAdminBot
        } else {
            ReportProcessingState::ProcessedByAdmin
        };

        update(common_report)
            .filter(id.eq(report_id))
            .set((
                processed_by_account_id.eq(moderator_id.as_db_id()),
                processing_state.eq(state),
                processing_state_change_unix_time.eq(time),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
