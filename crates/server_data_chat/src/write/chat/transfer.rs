use database_chat::current::write::{
    GetDbWriteCommandsChat, chat::transfer::TransferBudgetCheckResult,
};
use model_chat::AccountIdInternal;
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsChatTransfer);

impl WriteCommandsChatTransfer<'_> {
    pub async fn update_transfer_budget(
        &self,
        id: AccountIdInternal,
        actual_bytes_transferred: i64,
        yearly_limit_bytes: i64,
    ) -> Result<TransferBudgetCheckResult, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().transfer().update_transfer_budget(
                id,
                actual_bytes_transferred,
                yearly_limit_bytes,
            )
        })
    }
}
