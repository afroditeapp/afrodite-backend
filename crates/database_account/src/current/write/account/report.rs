use database::{
    DieselDatabaseError, current::write::GetDbWriteCommandsCommon, define_current_write_commands,
};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::{
    AccountIdInternal, CustomReportTypeNumberValue, ReportProcessingState, ReportTypeNumberInternal,
};
use simple_backend_utils::db::MyRunQueryDsl;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountReport);

impl CurrentWriteAccountReport<'_> {
    pub fn insert_custom_report_empty(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        custom_report_type_number: CustomReportTypeNumberValue,
    ) -> Result<(), DieselDatabaseError> {
        self.write().common().report().insert_report_content(
            creator,
            target,
            ReportTypeNumberInternal::CustomReport(custom_report_type_number),
            ReportProcessingState::Waiting,
        )?;

        Ok(())
    }

    pub fn upsert_custom_reports_file_hash(
        &mut self,
        sha256_custom_reports_file_hash: &str,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::custom_reports_file_hash::dsl::*;

        insert_into(custom_reports_file_hash)
            .values((
                row_type.eq(0),
                sha256_hash.eq(sha256_custom_reports_file_hash),
            ))
            .on_conflict(row_type)
            .do_update()
            .set(sha256_hash.eq(sha256_custom_reports_file_hash))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
