use database_account::current::write::GetDbWriteCommandsAccount;
use model::AccountIdInternal;
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsAccountLockAdmin);

impl WriteCommandsAccountLockAdmin<'_> {
    pub async fn set_locked_state(
        &self,
        id: AccountIdInternal,
        locked: bool,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().login().set_locked_state(id, locked)?;
            Ok(())
        })
    }
}
