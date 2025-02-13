use database_chat::current::{read::GetDbReadCommandsChat, write::GetDbWriteCommandsChat};
use model::AccountIdInternal;
use model_chat::ChatReportContent;
use server_data::{
    define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
    DataError,
};

define_cmd_wrapper_write!(WriteCommandsChatReport);

impl WriteCommandsChatReport<'_> {
    pub async fn process_report(
        &self,
        moderator_id: AccountIdInternal,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        content: ChatReportContent,
    ) -> Result<(), DataError> {
        let current_report = self
            .db_read(move |mut cmds| cmds.chat().report().get_report(creator, target))
            .await?;
        if current_report.content != content {
            return Err(DataError::NotAllowed.report());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.chat_admin()
                .report()
                .mark_report_done(moderator_id, creator, target)?;
            Ok(())
        })?;

        Ok(())
    }
}
