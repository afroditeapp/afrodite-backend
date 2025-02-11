use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ReportProcessingState};
use model_media::{MediaReport, MediaReportContentRaw};

define_current_read_commands!(CurrentReadMediaReport);

impl CurrentReadMediaReport<'_> {
    pub fn get_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<MediaReport, DieselDatabaseError> {
        use crate::schema::media_report::dsl::*;

        let report: Option<(ReportProcessingState, MediaReportContentRaw)> = media_report
            .filter(creator_account_id.eq(creator.as_db_id()))
            .filter(target_account_id.eq(target.as_db_id()))
            .select((processing_state, MediaReportContentRaw::as_select()))
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        let report = if let Some((state, content)) = report {
            MediaReport {
                processing_state: state,
                profile_content: content.iter().collect(),
            }
        } else {
            MediaReport::default()
        };

        Ok(report)
    }
}
