use database::current::read::GetDbReadCommandsCommon;
use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::{EventToClientInternal, UnixTime};
use model_account::{AccountIdInternal, EmailAddress, EmailMessages, EmailSendingState};
use server_data::{
    DataError,
    app::{EventManagerProvider, GetConfig, GetEmailSender},
    db_transaction, define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
};

use crate::write::GetWriteCommandsAccount;

pub enum TokenCheckResult {
    Valid,
    Invalid,
}

define_cmd_wrapper_write!(WriteCommandsAccountEmail);

impl WriteCommandsAccountEmail<'_> {
    pub async fn inital_setup_account_email_change(
        &self,
        id: AccountIdInternal,
        email: EmailAddress,
    ) -> Result<(), DataError> {
        self.0
            .account()
            .update_syncable_account_data(id, None, |_, _, _, verified| {
                *verified = false;
                Ok(())
            })
            .await?;
        db_transaction!(self, move |mut cmds| {
            cmds.account().data().update_account_email(id, &email)
        })
    }

    pub async fn set_email_verification_token(
        &self,
        id: AccountIdInternal,
        token: Vec<u8>,
        token_unix_time: UnixTime,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .email()
                .set_email_verification_token(id, token, token_unix_time)
        })
    }

    pub async fn verify_email_with_token(
        &self,
        token: Vec<u8>,
    ) -> Result<TokenCheckResult, DataError> {
        let token_validity_duration = self
            .config()
            .limits_account()
            .email_verification_token_validity_duration;

        let account_id = self
            .db_read(move |mut cmds| {
                let token_info = cmds
                    .account()
                    .email()
                    .find_account_by_email_verification_token(token)?;

                let Some((account_id, token_unix_time)) = token_info else {
                    return Ok(None);
                };

                if token_unix_time.duration_value_elapsed(token_validity_duration) {
                    return Ok(None);
                }

                Ok(Some(account_id))
            })
            .await?;

        if let Some(account_id) = account_id {
            let account = self
                .db_read(move |mut cmds| cmds.common().account(account_id))
                .await?;
            if !account.email_verified() {
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
            }
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

    pub async fn send_email_verification_message_high_priority(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        self.email_sender()
            .send_high_priority(id, EmailMessages::EmailVerification)
            .await
            .map_err(|_| DataError::EmailSendingFailed.report())?;

        Ok(())
    }

    pub async fn init_email_change(
        &self,
        id: AccountIdInternal,
        new_email: EmailAddress,
    ) -> Result<(), DataError> {
        let current_time = UnixTime::current_time();
        let (_, verification_token_bytes) = model::AccessToken::generate_new_with_bytes();

        db_transaction!(self, move |mut cmds| {
            cmds.account().email().init_email_change(
                id,
                new_email.0.clone(),
                current_time,
                verification_token_bytes,
            )
        })
    }

    pub async fn email_change_try_to_verify_new_email(
        &self,
        token: Vec<u8>,
    ) -> Result<TokenCheckResult, DataError> {
        let token_validity_duration = self
            .config()
            .limits_account()
            .email_change_min_wait_duration;

        let account_id = self
            .db_read(move |mut cmds| {
                let token_info = cmds
                    .account()
                    .email()
                    .find_account_by_email_change_verification_token(token)?;

                let Some((account_id, token_unix_time)) = token_info else {
                    return Ok(None);
                };

                if token_unix_time.duration_value_elapsed(token_validity_duration) {
                    return Ok(None);
                }

                Ok(Some(account_id))
            })
            .await?;

        if let Some(account_id) = account_id {
            db_transaction!(self, move |mut cmds| {
                let current_state = cmds.read().account().data().account_internal(account_id)?;
                if current_state.email_change_verified {
                    // Already verified
                    return Ok(());
                }
                cmds.account()
                    .email()
                    .verify_pending_email_address(account_id)
            })?;
            Ok(TokenCheckResult::Valid)
        } else {
            Ok(TokenCheckResult::Invalid)
        }
    }

    pub async fn cancel_email_change(&self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().email().clear_email_change_data(id)
        })
    }

    pub async fn send_email_change_verification_high_priority(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        self.email_sender()
            .send_high_priority(id, EmailMessages::EmailChangeVerification)
            .await
            .map_err(|_| DataError::EmailSendingFailed.report())?;

        Ok(())
    }

    pub async fn send_email_change_notification_high_priority(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        self.email_sender()
            .send_high_priority(id, EmailMessages::EmailChangeNotification)
            .await
            .map_err(|_| DataError::EmailSendingFailed.report())?;

        Ok(())
    }

    /// The new_email must be verified email address.
    pub async fn complete_email_change(
        &self,
        id: AccountIdInternal,
        new_email: EmailAddress,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .email()
                .complete_email_change(id, new_email.0)
        })?;

        self.0
            .account()
            .update_syncable_account_data(id, None, |_, _, _, email_verified| {
                *email_verified = true;
                Ok(())
            })
            .await?;

        self.event_manager()
            .send_connected_event(id, EventToClientInternal::AccountStateChanged)
            .await?;

        Ok(())
    }

    pub async fn clear_email_verification_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().email().clear_email_verification_token(id)
        })
    }

    pub async fn set_email_login_token(&self, id: AccountIdInternal) -> Result<(), DataError> {
        let current_time = UnixTime::current_time();
        let (_, token_bytes) = model::AccessToken::generate_new_with_bytes();

        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .email()
                .set_email_login_token(id, token_bytes, current_time)
        })
    }

    pub async fn send_email_login_token_high_priority(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        self.email_sender()
            .send_high_priority(id, EmailMessages::EmailLoginToken)
            .await
            .map_err(|_| DataError::EmailSendingFailed.report())?;

        Ok(())
    }

    pub async fn verify_email_login_token_and_invalidate(
        &self,
        token: Vec<u8>,
    ) -> Result<Option<AccountIdInternal>, DataError> {
        let token_validity_duration = self
            .config()
            .limits_account()
            .email_login_token_validity_duration;

        let account_id = self
            .db_read(move |mut cmds| {
                let token_info = cmds
                    .account()
                    .email()
                    .find_account_by_email_login_token(token.clone())?;

                let Some((account_id, token_unix_time)) = token_info else {
                    return Ok(None);
                };

                if token_unix_time.duration_value_elapsed(token_validity_duration) {
                    return Ok(None);
                }

                Ok(Some(account_id))
            })
            .await?;

        if let Some(account_id) = account_id {
            db_transaction!(self, move |mut cmds| {
                cmds.account().email().clear_email_login_token(account_id)
            })?;
            Ok(Some(account_id))
        } else {
            Ok(None)
        }
    }

    pub async fn clear_email_login_token(&self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().email().clear_email_login_token(id)
        })
    }
}
