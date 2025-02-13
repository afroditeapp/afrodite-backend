use database_chat::current::read::GetDbReadCommandsChat;
use model::AccountIdInternal;
use model_chat::ChatReport;
use server_data::{define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError};

define_cmd_wrapper_read!(ReadCommandsChatReport);

impl ReadCommandsChatReport<'_> {
    pub async fn get_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<ChatReport, DataError> {
        self.db_read(move |mut cmds| cmds.chat().report().get_report(creator, target))
            .await
            .into_error()
    }
}
