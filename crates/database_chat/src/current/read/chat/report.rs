use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ReportProcessingState};
use model_chat::{ChatReport, ChatReportContent};

define_current_read_commands!(CurrentReadChatReport);

impl CurrentReadChatReport<'_> {
    pub fn get_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<ChatReport, DieselDatabaseError> {
        use crate::schema::chat_report::dsl::*;

        let report: Option<(ReportProcessingState, ChatReportContent)> = chat_report
            .filter(creator_account_id.eq(creator.as_db_id()))
            .filter(target_account_id.eq(target.as_db_id()))
            .select((processing_state, ChatReportContent::as_select()))
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        let report = if let Some((state, content)) = report {
            ChatReport {
                processing_state: state,
                content,
            }
        } else {
            ChatReport::default()
        };

        Ok(report)
    }
}
