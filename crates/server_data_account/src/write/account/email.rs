use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::{EventToClientInternal, UnixTime};
use model_account::{
    AccountIdInternal, AccountInternal, EmailAddress, EmailMessages, EmailSendingState,
};
use server_data::{
    DataError,
    app::{EventManagerProvider, GetEmailSender},
    db_transaction, define_cmd_wrapper_write,
    read::DbRead,
    result::Result,
    write::DbTransaction,
};
use simple_backend_utils::time::DurationValue;

use crate::write::GetWriteCommandsAccount;

pub enum TokenCheckResult {
    Valid,
    Invalid,
}

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

    pub async fn set_email_confirmation_token(
        &self,
        id: AccountIdInternal,
        token: Vec<u8>,
        token_unix_time: UnixTime,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .email()
                .set_email_confirmation_token(id, token, token_unix_time)
        })
    }

    pub async fn confirm_email_with_token(
        &self,
        token: Vec<u8>,
    ) -> Result<TokenCheckResult, DataError> {
        let account_id = self
            .db_read(move |mut cmds| {
                let account_data = cmds
                    .account()
                    .email()
                    .find_account_by_email_confirmation_token(token)?;

                let Some((account_id, token_unix_time)) = account_data else {
                    return Ok(None);
                };

                if token_unix_time.duration_value_elapsed(DurationValue::from_seconds(
                    AccountInternal::EMAIL_CONFIRMATION_TOKEN_VALIDITY_SECONDS,
                )) {
                    return Ok(None);
                }

                Ok(Some(account_id))
            })
            .await?;

        if let Some(account_id) = account_id {
            self.0
                .account()
                .update_syncable_account_data(account_id, None, |_, _, _, email_verified| {
                    *email_verified = true;
                    Ok(())
                })
                .await?;
            self.event_manager()
                .send_connected_event(account_id, EventToClientInternal::AccountStateChanged)
                .await?;
            db_transaction!(self, move |mut cmds| {
                cmds.account()
                    .email()
                    .clear_email_confirmation_token(account_id)?;
                Ok(())
            })?;
            Ok(TokenCheckResult::Valid)
        } else {
            Ok(TokenCheckResult::Invalid)
        }
    }

    pub async fn send_email_if_not_already_sent(
        &self,
        id: AccountIdInternal,
        email: EmailMessages,
    ) -> Result<(), DataError> {
        self.send_email_internal(id, email, false).await
    }

    pub async fn send_email_if_sending_is_not_in_progress(
        &self,
        id: AccountIdInternal,
        email: EmailMessages,
    ) -> Result<(), DataError> {
        self.send_email_internal(id, email, true).await
    }

    async fn send_email_internal(
        &self,
        id: AccountIdInternal,
        email: EmailMessages,
        send_again: bool,
    ) -> Result<(), DataError> {
        let send_needed = db_transaction!(self, move |mut cmds| {
            let mut send_needed = false;
            cmds.account()
                .email()
                .modify_email_sending_states(id, |state| {
                    let correct_field = state.get_ref_mut_to(email);
                    if *correct_field == EmailSendingState::NotSent
                        || (send_again && *correct_field == EmailSendingState::SentSuccessfully)
                    {
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
