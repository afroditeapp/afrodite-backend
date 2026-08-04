pub mod custom;

use std::sync::Arc;

use error_stack::ResultExt;
use model::{AccessToken, AccountIdInternal, EmailMessages, EventToClientInternal, UnixTime};
use server_api::{
    app::{GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_data::{
    DataError,
    email::{CustomEmailMsg, EmailData, EmailError, HighPriorityEmailMsg, NormalEmailMsg},
    read::GetReadCommandsCommon,
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_state::S;
use simple_backend::email::SmtpClient;
use simple_backend_config::SimpleBackendConfig;
use tokio::sync::mpsc::Receiver;
use tracing::{error, warn};

use self::custom::CustomEmailHandler;
use crate::ServerQuitWatcher;

pub struct EmailManagerQuitHandle {
    task: tokio::task::JoinHandle<()>,
}

impl EmailManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("EmailManagerQuitHandle quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct EmailManager {
    state: S,
    smtp_client: Arc<SmtpClient>,
    config: Arc<SimpleBackendConfig>,
    normal_receiver: Receiver<NormalEmailMsg>,
    high_priority_receiver: Receiver<HighPriorityEmailMsg>,
}

impl EmailManager {
    pub fn new_manager(
        state: S,
        smtp_client: SmtpClient,
        config: Arc<SimpleBackendConfig>,
        mut quit_notification: ServerQuitWatcher,
        normal_receiver: Receiver<NormalEmailMsg>,
        high_priority_receiver: Receiver<HighPriorityEmailMsg>,
        custom_receiver: Receiver<CustomEmailMsg>,
    ) -> EmailManagerQuitHandle {
        let smtp_client = Arc::new(smtp_client);
        let custom_handler =
            CustomEmailHandler::new(state.clone(), smtp_client.clone(), custom_receiver);

        EmailManagerQuitHandle {
            task: tokio::spawn(async move {
                let mut manager = EmailManager {
                    state,
                    smtp_client,
                    config,
                    normal_receiver,
                    high_priority_receiver,
                };

                tokio::select! {
                    _ = quit_notification.recv() => (),
                    _ = manager.run(custom_handler) => (),
                }

                // Save email limit state on quit
                manager.smtp_client.save_state(&manager.config).await;
            }),
        }
    }

    async fn run(&mut self, custom_handler: CustomEmailHandler) {
        let custom_email_handling = custom_handler.run();
        tokio::pin!(custom_email_handling);

        loop {
            tokio::select! {
                biased;

                Some(cmd) = self.high_priority_receiver.recv() => {
                    match cmd {
                        HighPriorityEmailMsg::Normal { recipient, message, result_sender } => {
                            let result = self.handle_send(recipient, message).await;

                            if let Err(e) = &result {
                                error!("High priority email send failed: {:?}", e);
                            }

                            let _ = result_sender.send(result.map_err(|_| DataError::EmailSendingFailed));
                        }
                        HighPriorityEmailMsg::RegistrationToken { email, token, result_sender } => {
                            let result = self.handle_send_registration_token(&email, &token).await;
                            if let Err(e) = &result {
                                error!("Registration token email send failed: {:?}", e);
                            }
                            let _ = result_sender.send(result.map_err(|_| DataError::EmailSendingFailed));
                        }
                    }
                }
                Some(cmd) = self.normal_receiver.recv() => {
                    if let Err(e) = self.handle_send(cmd.recipient, cmd.message).await {
                        error!("Email send failed: {:?}", e);
                    }
                }
                _ = &mut custom_email_handling => {
                    break;
                }
                else => {
                    warn!("Email channel closed");
                    break;
                }
            }
        }
    }

    async fn handle_send(
        &self,
        recipient: AccountIdInternal,
        message: EmailMessages,
    ) -> error_stack::Result<(), EmailError> {
        let Some(info) = self.get_email_data(recipient, message).await? else {
            // Email disabled for the email recipient
            return Ok(());
        };

        self.smtp_client
            .send(
                &info.email_address,
                &info.subject,
                &info.body,
                info.body_is_html,
            )
            .await
            .change_context(EmailError::SendingFailed)?;

        self.mark_as_sent(recipient, message).await
    }

    async fn handle_send_registration_token(
        &self,
        email: &str,
        token: &str,
    ) -> error_stack::Result<(), EmailError> {
        if email.ends_with("@example.com") {
            return Ok(());
        }

        let email_content = self.state.config().email_content();
        // TODO(prod): Get language using login route
        let getter = email_content.get::<String>(None);
        let content = getter
            .email_login(token)
            .change_context(EmailError::GettingEmailDataFailed)?;

        self.smtp_client
            .send(email, &content.subject, &content.body, content.body_is_html)
            .await
            .change_context(EmailError::SendingFailed)?;

        Ok(())
    }

    /// If `Ok(None)` is returned the email sending is disabled for the
    /// provided `recipient`.
    async fn get_email_data(
        &self,
        recipient: AccountIdInternal,
        message: EmailMessages,
    ) -> error_stack::Result<Option<EmailData>, EmailError> {
        let data = self
            .state
            .read()
            .account()
            .email_address_state(recipient)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        // For email change verification, use the new email address
        let email_to_use = if message == EmailMessages::EmailChangeVerification {
            data.email_change.clone()
        } else {
            data.email.clone()
        };

        let email = if let Some(email) = email_to_use {
            if email.0.ends_with("@example.com") {
                let is_bot = async || {
                    self.state
                        .read()
                        .common()
                        .is_bot(recipient)
                        .await
                        .map_err(|e| e.into_report())
                        .change_context(EmailError::GettingEmailDataFailed)
                };
                if message == EmailMessages::EmailVerification
                    && (self.state.config().debug_mode() || is_bot().await?)
                {
                    db_write_raw!(self.state, move |cmds| {
                        cmds.account()
                            .update_syncable_account_data(recipient, |account| {
                                account.email_verified = true;
                                Ok(())
                            })
                            .await?;
                        cmds.events()
                            .send_connected_event(
                                recipient,
                                EventToClientInternal::AccountStateChanged,
                            )
                            .await?;
                        Ok(())
                    })
                    .await
                    .map_err(|e| e.into_report())
                    .change_context(EmailError::GettingEmailDataFailed)?;
                }

                self.mark_as_sent(recipient, message).await?;

                return Ok(None);
            } else {
                email.0
            }
        } else {
            return Ok(None);
        };

        let email_content = self.state.config().email_content();

        let language = self
            .state
            .read()
            .common()
            .client_config()
            .client_language(recipient)
            .await
            .ok()
            .flatten();

        let getter = email_content.get(language.as_ref());

        let content = match message {
            EmailMessages::EmailVerification => {
                let token = self
                    .generate_token_for_email_verification(recipient)
                    .await?;
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
                    .get_token_for_email_change_verification(recipient)
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
        recipient: AccountIdInternal,
        message: EmailMessages,
    ) -> error_stack::Result<(), EmailError> {
        db_write_raw!(self.state, move |cmds| {
            cmds.account()
                .email()
                .mark_email_as_sent(recipient, message)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(EmailError::MarkAsSentFailed)?;

        Ok(())
    }

    async fn generate_token_for_email_verification(
        &self,
        recipient: AccountIdInternal,
    ) -> error_stack::Result<String, EmailError> {
        let token_and_time = self
            .state
            .read()
            .account()
            .email_verification_token(recipient)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        let current_time = UnixTime::current_time();

        // Reuse existing valid token to avoid sending multiple emails
        // with different links in a short time period.
        let (token, token_bytes) =
            if let (Some(existing_token_bytes), Some(token_time)) = token_and_time {
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
                .set_email_verification_token(recipient, token_bytes, current_time)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(EmailError::GettingEmailDataFailed)?;

        Ok(token.into_string())
    }

    async fn get_token_for_email_change_verification(
        &self,
        recipient: AccountIdInternal,
    ) -> error_stack::Result<String, EmailError> {
        let email_change = self
            .state
            .read()
            .account()
            .email_change(recipient)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        if let Some(email_change) = email_change {
            let token = AccessToken::from_bytes(&email_change.email_change_verification_token);
            Ok(token.into_string())
        } else {
            Err(EmailError::GettingEmailDataFailed)
                .attach_printable("No email change verification token found")
        }
    }
}
