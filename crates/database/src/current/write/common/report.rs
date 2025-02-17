use diesel::{delete, insert_into, prelude::*};
use error_stack::Result;
use model::{AccountIdInternal, ReportIdDb, ReportProcessingState, ReportTypeNumber, UnixTime};
use simple_backend_utils::current_unix_time;

use crate::{define_current_read_commands, DieselDatabaseError, IntoDatabaseError};

define_current_read_commands!(CurrentWriteCommonReport);

impl CurrentWriteCommonReport<'_> {
    pub fn insert_report_content(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        type_number: ReportTypeNumber,
        state: ReportProcessingState,
    ) -> Result<ReportIdDb, DieselDatabaseError> {
        use model::schema::common_report::dsl::*;

        let time = current_unix_time();

        let db_id: ReportIdDb = insert_into(common_report)
            .values((
                creator_account_id.eq(creator.as_db_id()),
                target_account_id.eq(target.as_db_id()),
                report_type_number.eq(type_number),
                creation_unix_time.eq(time),
                processing_state.eq(state),
                processing_state_change_unix_time.eq(time),
            ))
            .returning(id)
            .get_result(self.conn())
            .into_db_error((creator, target))?;

        Ok(db_id)
    }

    pub fn delete_old_reports(
        &mut self,
        type_number: ReportTypeNumber,
        deletion_allowed_time: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_report::dsl::*;

        delete(common_report)
            .filter(report_type_number.eq(type_number))
            .filter(processing_state.eq(ReportProcessingState::Done))
            .filter(processing_state_change_unix_time.ge(deletion_allowed_time))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
