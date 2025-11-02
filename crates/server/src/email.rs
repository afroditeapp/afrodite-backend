use error_stack::ResultExt;
use model::{AccessToken, AccountIdInternal, EmailMessages, EventToClientInternal, UnixTime};
use server_api::{
    app::{GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_data::read::GetReadCommandsCommon;
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_state::S;
use simple_backend::email::{EmailData, EmailDataProvider, EmailError};

pub struct ServerEmailDataProvider {
    state: S,
}

impl ServerEmailDataProvider {
    pub fn new(state: S) -> Self {
        Self { state }
    }
}

impl EmailDataProvider<AccountIdInternal, EmailMessages> for ServerEmailDataProvider {
    async fn get_email_data(
        &self,
        receiver: AccountIdInternal,
        message: EmailMessages,
    ) -> error_stack::Result<Option<EmailData>, simple_backend::email::EmailError> {
        let data = self
            .state
            .read()
            .account()
            .account_data(receiver)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        // For email change verification, use the new email address
        let email_to_use = if message == EmailMessages::EmailChangeVerification {
            data.change_email.clone()
        } else {
            data.email.clone()
        };

        let email = if let Some(email) = email_to_use {
            let treat_example_com_as_test =
                if let Some(email_config) = self.state.config().simple_backend().email_sending() {
                    !email_config.debug_example_com_is_normal_email
                } else {
                    true
                };

            if treat_example_com_as_test && email.0.ends_with("@example.com") {
                if message == EmailMessages::EmailVerification {
                    db_write_raw!(self.state, move |cmds| {
                        cmds.account()
                            .update_syncable_account_data(
                                receiver,
                                None,
                                |_, _, _, email_verified| {
                                    *email_verified = true;
                                    Ok(())
                                },
                            )
                            .await?;
                        cmds.events()
                            .send_connected_event(
                                receiver,
                                EventToClientInternal::AccountStateChanged,
                            )
                            .await?;
                        Ok(())
                    })
                    .await
                    .map_err(|e| e.into_report())
                    .change_context(EmailError::GettingEmailDataFailed)?;
                }

                self.mark_as_sent(receiver, message).await?;

                return Ok(None);
            }

            email.0
        } else {
            return Ok(None);
        };

        let email_content = self
            .state
            .config()
            .email_content()
            .ok_or(EmailError::GettingEmailDataFailed)
            .attach_printable("Email content not configured")?;

        let language = self
            .state
            .read()
            .common()
            .client_config()
            .client_language(receiver)
            .await
            .ok()
            .flatten();

        let getter = email_content.get(language.as_ref());

        let content = match message {
            EmailMessages::EmailVerification => {
                let token = self.generate_token_for_email_verification(receiver).await?;
                getter.email_verification(&token)
            }
            EmailMessages::NewMessage => getter.new_message(),
            EmailMessages::NewLike => getter.new_like(),
            EmailMessages::AccountDeletionRemainderFirst => {
                getter.account_deletion_remainder_first()
            }
            EmailMessages::AccountDeletionRemainderSecond => {
                getter.account_deletion_remainder_second()
            }
            EmailMessages::AccountDeletionRemainderThird => {
                getter.account_deletion_remainder_third()
            }
            EmailMessages::EmailChangeVerification => {
                let token = self
                    .get_token_for_email_change_verification(receiver)
                    .await?;
                getter.email_change_verification(&token)
            }
            EmailMessages::EmailChangeNotification => getter.email_change_notification(),
        }
        .change_context(EmailError::GettingEmailDataFailed)?;

        let email_data = EmailData {
            email_address: email,
            subject: content.subject,
            body: content.body,
            body_is_html: content.body_is_html,
        };

        Ok(Some(email_data))
    }

    async fn mark_as_sent(
        &self,
        receiver: AccountIdInternal,
        message: EmailMessages,
    ) -> error_stack::Result<(), simple_backend::email::EmailError> {
        db_write_raw!(self.state, move |cmds| {
            cmds.account()
                .email()
                .mark_email_as_sent(receiver, message)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(EmailError::MarkAsSentFailed)
    }
}

impl ServerEmailDataProvider {
    async fn generate_token_for_email_verification(
        &self,
        receiver: AccountIdInternal,
    ) -> error_stack::Result<String, simple_backend::email::EmailError> {
        let account_internal = self
            .state
            .read()
            .account()
            .account_internal(receiver)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        let current_time = UnixTime::current_time();

        // Reuse existing valid token to avoid sending multiple emails
        // with different links in a short time period.
        let (token, token_bytes) = if let (Some(existing_token_bytes), Some(token_time)) = (
            account_internal.email_verification_token,
            account_internal.email_verification_token_unix_time,
        ) {
            if token_time.duration_value_elapsed(
                self.state
                    .config()
                    .limits_account()
                    .email_verification_token_validity_duration,
            ) {
                AccessToken::generate_new_with_bytes()
            } else {
                (
                    AccessToken::from_bytes(&existing_token_bytes),
                    existing_token_bytes,
                )
            }
        } else {
            AccessToken::generate_new_with_bytes()
        };

        db_write_raw!(self.state, move |cmds| {
            cmds.account()
                .email()
                .set_email_verification_token(receiver, token_bytes, current_time)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(EmailError::GettingEmailDataFailed)?;

        Ok(token.into_string())
    }

    async fn get_token_for_email_change_verification(
        &self,
        receiver: AccountIdInternal,
    ) -> error_stack::Result<String, simple_backend::email::EmailError> {
        let account_internal = self
            .state
            .read()
            .account()
            .account_internal(receiver)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        if let Some(token_bytes) = account_internal.change_email_verification_token {
            let token = AccessToken::from_bytes(&token_bytes);
            Ok(token.into_string())
        } else {
            Err(EmailError::GettingEmailDataFailed)
                .attach_printable("No email change verification token found")
        }
    }
}
