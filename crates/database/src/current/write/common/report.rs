use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::{AccountIdInternal, ReportIdDb, ReportProcessingState, ReportTypeNumber};
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
                content_edit_unix_time.eq(time),
                processing_state.eq(state),
                processing_state_change_unix_time.eq(time),
            ))
            .returning(id)
            .get_result(self.conn())
            .into_db_error((creator, target))?;

        Ok(db_id)
    }
}
