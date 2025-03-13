use database::{current::write::GetDbWriteCommandsCommon, define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, ReportProcessingState, ReportTypeNumberInternal};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileReport);

impl CurrentWriteProfileReport<'_> {
    pub fn insert_profile_name_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        name: String,
    ) -> Result<(), DieselDatabaseError> {
        let id = self.write().common().report().insert_report_content(
            creator,
            target,
            ReportTypeNumberInternal::ProfileName,
            ReportProcessingState::Waiting
        )?;

        {
            use model::schema::profile_report_profile_name::dsl::*;

            insert_into(profile_report_profile_name)
                .values((
                    report_id.eq(id),
                    profile_name.eq(&name),
                ))
                .execute(self.conn())
                .into_db_error((creator, target))?;
        }

        Ok(())
    }

    pub fn insert_profile_text_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        text: String,
    ) -> Result<(), DieselDatabaseError> {
        let id = self.write().common().report().insert_report_content(
            creator,
            target,
            ReportTypeNumberInternal::ProfileText,
            ReportProcessingState::Waiting
        )?;

        {
            use model::schema::profile_report_profile_text::dsl::*;

            insert_into(profile_report_profile_text)
                .values((
                    report_id.eq(id),
                    profile_text.eq(&text),
                ))
                .execute(self.conn())
                .into_db_error((creator, target))?;
        }

        Ok(())
    }
}
