use database::{current::write::GetDbWriteCommandsCommon, define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, ReportProcessingState, ReportTypeNumber};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteChatReport);

impl CurrentWriteChatReport<'_> {
    pub fn insert_chat_message_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        message: String,
    ) -> Result<(), DieselDatabaseError> {
        let id = self.write().common().report().insert_report_content(
            creator,
            target,
            ReportTypeNumber::ChatMessage,
            ReportProcessingState::Waiting,
        )?;

        {
            use model::schema::chat_report_chat_message::dsl::*;

            insert_into(chat_report_chat_message)
                .values((
                    report_id.eq(id),
                    chat_message.eq(message),
                ))
                .execute(self.conn())
                .into_db_error((creator, target))?;
        }

        Ok(())
    }
}
