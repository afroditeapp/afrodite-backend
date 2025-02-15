use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, ReportIdDb, ReportTypeNumber};

use crate::{define_current_read_commands, DieselDatabaseError, IntoDatabaseError};

define_current_read_commands!(CurrentReadCommonReport);

impl CurrentReadCommonReport<'_> {
    pub fn get_report_id(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        type_number: ReportTypeNumber,
    ) -> Result<Option<ReportIdDb>, DieselDatabaseError> {
        use model::schema::common_report::dsl::*;

        let db_id: Option<ReportIdDb> = common_report
            .filter(creator_account_id.eq(creator.as_db_id()))
            .filter(target_account_id.eq(target.as_db_id()))
            .filter(report_type_number.eq(type_number))
            .select(id)
            .first(self.conn())
            .optional()
            .into_db_error((creator, target))?;

        Ok(db_id)
    }
}
