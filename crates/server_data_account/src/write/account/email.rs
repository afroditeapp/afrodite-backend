use database_account::current::write::GetDbWriteCommandsAccount;
use model_account::{AccountIdInternal, EmailAddress, EmailMessages, EmailSendingState};
use server_data::{
    app::GetEmailSender, define_cmd_wrapper_write, result::Result, write::DbTransaction, DataError,
};

define_cmd_wrapper_write!(WriteCommandsAccountEmail);

impl WriteCommandsAccountEmail<'_> {
    pub async fn account_email(
        &self,
        id: AccountIdInternal,
        email: EmailAddress,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().data().update_account_email(id, &email)
        })
    }

    pub async fn send_email_if_not_already_sent(
        &self,
        id: AccountIdInternal,
        email: EmailMessages,
    ) -> Result<(), DataError> {
        let send_needed = db_transaction!(self, move |mut cmds| {
            let mut send_needed = false;
            cmds.account()
                .email()
                .modify_email_sending_states(id, |state| {
                    let correct_field = state.get_ref_mut_to(email);
                    if *correct_field == EmailSendingState::NotSent {
                        *correct_field = EmailSendingState::SendRequested;
                        send_needed = true;
                    }
                })?;
            Ok(send_needed)
        })?;

        if send_needed {
            self.email_sender().send(id, email);
        }

        Ok(())
    }

    pub async fn mark_email_as_sent(
        &self,
        id: AccountIdInternal,
        email: EmailMessages,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .email()
                .modify_email_sending_states(id, |state| {
                    let correct_field = state.get_ref_mut_to(email);
                    *correct_field = EmailSendingState::SentSuccessfully;
                })
        })?;

        Ok(())
    }
}
