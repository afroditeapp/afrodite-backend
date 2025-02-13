use database_chat::current::read::GetDbReadCommandsChat;
use model_chat::GetChatReportList;
use server_data::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsChatReport);

impl ReadCommandsChatReport<'_> {
    pub async fn get_report_list(
        &self,
    ) -> Result<GetChatReportList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat_admin()
                .report()
                .report_list()
        })
        .await
        .into_error()
    }
}
