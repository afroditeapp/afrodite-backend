use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ReportProcessingState};
use model_profile::ProfileReport;

define_current_read_commands!(CurrentReadProfileReport);

impl CurrentReadProfileReport<'_> {
    pub fn profile_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<ProfileReport, DieselDatabaseError> {
        use crate::schema::profile_report::dsl::*;

        let report: Option<(ReportProcessingState, Option<String>)> = profile_report
            .filter(creator_account_id.eq(creator.as_db_id()))
            .filter(target_account_id.eq(target.as_db_id()))
            .select((processing_state, profile_text))
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        let report = if let Some((state, text)) = report {
            ProfileReport {
                processing_state: state,
                profile_text: text,
            }
        } else {
            ProfileReport::default()
        };

        Ok(report)
    }
}
