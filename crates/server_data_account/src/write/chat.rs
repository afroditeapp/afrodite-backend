use database_account::current::write::GetDbWriteCommandsAccount;
use model::AccountIdInternal;
use server_data::{define_cmd_wrapper_write, DataError, write::DbTransaction};
use server_common::result::Result;

define_cmd_wrapper_write!(WriteCommandsChatUtils);

impl WriteCommandsChatUtils<'_> {
    pub async fn remove_fcm_device_token_and_pending_notification_token(&self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_chat_utils()
                .remove_fcm_device_token_and_pending_notification_token(id)
        })?;

        Ok(())
    }
}
