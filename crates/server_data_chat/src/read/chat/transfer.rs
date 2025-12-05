use database_chat::current::read::{
    GetDbReadCommandsChat, chat::transfer::TransferBudgetCheckResult,
};
use model_chat::AccountIdInternal;
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsChatTransfer);

impl ReadCommandsChatTransfer<'_> {
    pub async fn check_transfer_budget(
        &self,
        id: AccountIdInternal,
        transfer_bytes: u32,
        yearly_limit_bytes: i64,
    ) -> Result<TransferBudgetCheckResult, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .transfer()
                .check_transfer_budget(id, transfer_bytes, yearly_limit_bytes)
        })
        .await
        .into_error()
    }
}
