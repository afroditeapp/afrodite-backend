use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, ReportProcessingState, UnixTime};
use model_account::AccountReportContent;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountReport);

impl CurrentWriteAccountReport<'_> {
    pub fn upsert_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        content: AccountReportContent,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_report::dsl::*;

        let time = UnixTime::current_time();

        let state = if content.is_empty() {
            ReportProcessingState::Empty
        } else {
            ReportProcessingState::Waiting
        };

        insert_into(account_report)
            .values((
                creator_account_id.eq(creator.as_db_id()),
                target_account_id.eq(target.as_db_id()),
                creation_unix_time.eq(time),
                content_edit_unix_time.eq(time),
                processing_state.eq(state),
                processing_state_change_unix_time.eq(time),
                &content,
            ))
            .on_conflict((creator_account_id, target_account_id))
            .do_update()
            .set((
                content_edit_unix_time.eq(time),
                processing_state.eq(state),
                processing_state_change_unix_time.eq(time),
                &content,
            ))
            .execute(self.conn())
            .into_db_error((creator, target))?;

        Ok(())
    }

    pub fn upsert_custom_reports_file_hash(
        &mut self,
        sha256_custom_reports_file_hash: &str,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::custom_reports_file_hash::dsl::*;

        insert_into(custom_reports_file_hash)
            .values((row_type.eq(0), sha256_hash.eq(sha256_custom_reports_file_hash)))
            .on_conflict(row_type)
            .do_update()
            .set(sha256_hash.eq(sha256_custom_reports_file_hash))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
