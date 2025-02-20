use database::{current::write::GetDbWriteCommandsCommon, define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, ContentId, ReportProcessingState, ReportTypeNumber};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteMediaReport);

impl CurrentWriteMediaReport<'_> {
    pub fn insert_profile_content_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        content: ContentId,
    ) -> Result<(), DieselDatabaseError> {
        let id = self.write().common().report().insert_report_content(
            creator,
            target,
            ReportTypeNumber::ProfileContent,
            ReportProcessingState::Waiting,
        )?;

        {
            use model::schema::media_report_profile_content::dsl::*;

            insert_into(media_report_profile_content)
                .values((
                    report_id.eq(id),
                    profile_content_uuid.eq(&content),
                ))
                .execute(self.conn())
                .into_db_error((creator, target))?;
        }

        Ok(())
    }
}
