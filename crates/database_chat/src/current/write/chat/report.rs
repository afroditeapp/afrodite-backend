use database::{
    DieselDatabaseError, current::write::GetDbWriteCommandsCommon, define_current_write_commands,
};
use diesel::{ExpressionMethods, insert_into, prelude::*};
use error_stack::Result;
use model::{AccountIdInternal, ReportProcessingState, ReportTypeNumberInternal};
use model_chat::NewChatMessageReportInternal;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteChatReport);

impl CurrentWriteChatReport<'_> {
    pub fn insert_chat_message_report(
        &mut self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        message: NewChatMessageReportInternal,
    ) -> Result<(), DieselDatabaseError> {
        let id = self.write().common().report().insert_report_content(
            creator,
            target,
            ReportTypeNumberInternal::ChatMessage,
            ReportProcessingState::Waiting,
        )?;

        {
            use model::schema::chat_report_chat_message::dsl::*;

            insert_into(chat_report_chat_message)
                .values((report_id.eq(id), message))
                .execute(self.conn())
                .into_db_error((creator, target))?;
        }

        Ok(())
    }
}
