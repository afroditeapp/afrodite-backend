use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, ReportProcessingState};
use model_media::{MediaReportContent, MediaReportContentRaw};
use simple_backend_utils::{current_unix_time, ContextExt};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteMediaReport);

impl CurrentWriteMediaReport<'_> {
    pub fn upsert_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        content: MediaReportContent,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_report::dsl::*;

        let time = current_unix_time();

        let state = if content.profile_content.is_empty() {
            ReportProcessingState::Empty
        } else {
            ReportProcessingState::Waiting
        };

        let mut iter = content.profile_content.into_iter();
        let content_raw = MediaReportContentRaw {
            profile_content_uuid_0: iter.next(),
            profile_content_uuid_1: iter.next(),
            profile_content_uuid_2: iter.next(),
            profile_content_uuid_3: iter.next(),
            profile_content_uuid_4: iter.next(),
            profile_content_uuid_5: iter.next(),
        };
        if iter.next().is_some() {
            return Err(DieselDatabaseError::NotAllowed.report());
        }

        insert_into(media_report)
            .values((
                creator_account_id.eq(creator.as_db_id()),
                target_account_id.eq(target.as_db_id()),
                creation_unix_time.eq(time),
                content_edit_unix_time.eq(time),
                processing_state.eq(state),
                processing_state_change_unix_time.eq(time),
                &content_raw,
            ))
            .on_conflict((creator_account_id, target_account_id))
            .do_update()
            .set((
                content_edit_unix_time.eq(time),
                processing_state.eq(state),
                processing_state_change_unix_time.eq(time),
                &content_raw,
            ))
            .execute(self.conn())
            .into_db_error((creator, target))?;

        Ok(())
    }
}
