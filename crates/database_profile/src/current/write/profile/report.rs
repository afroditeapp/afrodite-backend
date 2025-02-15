use database::{current::write::GetDbWriteCommandsCommon, define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, ReportProcessingState, ReportTypeNumber};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileReport);

impl CurrentWriteProfileReport<'_> {
    pub fn upsert_profile_name_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        name: String,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_report_profile_name::dsl::*;

        let id = self.write().common().report().upsert_report_content(
            creator,
            target,
            ReportTypeNumber::ProfileName,
            ReportProcessingState::Waiting
        )?;

        insert_into(profile_report_profile_name)
            .values((
                report_id.eq(id),
                profile_name.eq(&name),
            ))
            .on_conflict(report_id)
            .do_update()
            .set((
                profile_name.eq(&name),
            ))
            .execute(self.conn())
            .into_db_error((creator, target))?;

        Ok(())
    }

    pub fn upsert_profile_text_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        text: String,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_report_profile_text::dsl::*;

        let id = self.write().common().report().upsert_report_content(
            creator,
            target,
            ReportTypeNumber::ProfileText,
            ReportProcessingState::Waiting
        )?;

        insert_into(profile_report_profile_text)
            .values((
                report_id.eq(id),
                profile_text.eq(&text),
            ))
            .on_conflict(report_id)
            .do_update()
            .set((
                profile_text.eq(&text),
            ))
            .execute(self.conn())
            .into_db_error((creator, target))?;

        Ok(())
    }
}
