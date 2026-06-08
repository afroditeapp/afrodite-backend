use database_account::current::write::GetDbWriteCommandsAccount;
use model::AccountIdInternal;
use model_account::{CustomEmailId, UpdateCustomEmail};
use server_data::{
    DataError, app::GetEmailSender, db_transaction, define_cmd_wrapper_write, result::Result,
    write::DbTransaction,
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

    pub async fn send_custom_email(
        &self,
        email_id: CustomEmailId,
        account_ids: Vec<AccountIdInternal>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .init_custom_email_sending(email_id, &account_ids)?;
            Ok(())
        })?;

        self.email_sender()
            .trigger_custom_email_sending(email_id.eid);

        Ok(())
    }

    pub async fn mark_custom_email_sent(
        &self,
        email_id: CustomEmailId,
        account_id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .mark_custom_email_sent(email_id, &account_id)?;
            Ok(())
        })
    }

    pub async fn mark_custom_email_sending_completed(
        &self,
        email_id: CustomEmailId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .set_custom_email_sending_completed(email_id)?;
            Ok(())
        })
    }
}
