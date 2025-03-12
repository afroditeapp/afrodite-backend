use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ReportProcessingState};
use model_account::{AccountReport, AccountReportContent};

define_current_read_commands!(CurrentReadAccountReport);

impl CurrentReadAccountReport<'_> {
    pub fn get_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<AccountReport, DieselDatabaseError> {
        use crate::schema::account_report::dsl::*;

        let report: Option<(ReportProcessingState, AccountReportContent)> = account_report
            .filter(creator_account_id.eq(creator.as_db_id()))
            .filter(target_account_id.eq(target.as_db_id()))
            .select((processing_state, AccountReportContent::as_select()))
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        let report = if let Some((state, content)) = report {
            AccountReport {
                processing_state: state,
                content,
            }
        } else {
            AccountReport::default()
        };

        Ok(report)
    }

    pub fn custom_reports_hash(&mut self) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::custom_reports_file_hash::dsl::*;

        custom_reports_file_hash
            .filter(row_type.eq(0))
            .select(sha256_hash)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
    }
}
