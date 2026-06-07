use database_account::current::write::GetDbWriteCommandsAccount;
use model::AccountIdInternal;
use model_account::{CustomEmailId, UpdateCustomEmail};
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsAccountCustomEmailAdmin);

impl WriteCommandsAccountCustomEmailAdmin<'_> {
    pub async fn create_custom_email(
        &self,
        id: AccountIdInternal,
    ) -> Result<CustomEmailId, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().custom_email().create_custom_email(id)
        })
    }

    pub async fn update_custom_email(&self, data: UpdateCustomEmail) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .update_custom_email(data)
        })
    }
}
