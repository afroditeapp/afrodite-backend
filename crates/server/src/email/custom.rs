use std::sync::Arc;

use config::file_email_content::EmailContent;
use error_stack::{ResultExt, report};
use model::AccountIdInternal;
use model_account::CustomEmailId;
use server_api::{
    app::{GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_data::{
    email::{CustomEmailMsg, EmailData, EmailError},
    read::GetReadCommandsCommon,
};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_state::S;
use simple_backend::email::SmtpClient;
use tokio::sync::mpsc::Receiver;
use tracing::{error, warn};

pub struct CustomEmailHandler {
    state: S,
    smtp_client: Arc<SmtpClient>,
    custom_receiver: Receiver<CustomEmailMsg>,
}

impl CustomEmailHandler {
    pub fn new(
        state: S,
        smtp_client: Arc<SmtpClient>,
        custom_receiver: Receiver<CustomEmailMsg>,
    ) -> Self {
        Self {
            state,
            smtp_client,
            custom_receiver,
        }
    }

    pub async fn run(mut self) {
        loop {
            let Some(next) = self.custom_receiver.recv().await else {
                warn!("Custom email channel closed");
                return;
            };
            match next {
                CustomEmailMsg::SendToAll { email_id } => {
                    let email_id = model_account::CustomEmailId::new(email_id);
                    if let Err(e) = self.send_unsent_custom_emails(email_id).await {
                        error!("Custom email sending failed: {:?}", e);
                    }
                }
                CustomEmailMsg::SendDraft {
                    email_id,
                    target_account_id,
                } => {
                    if let Err(e) = self.send_draft_to_target(target_account_id, email_id).await {
                        error!("Custom email draft sending failed: {:?}", e);
                    }
                }
            }
        }
    }

    async fn send_unsent_custom_emails(
        &self,
        email_id: CustomEmailId,
    ) -> error_stack::Result<(), EmailError> {
        let unsent = self
            .state
            .read()
            .account_admin()
            .custom_email()
            .custom_email_unsent_accounts(email_id)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        for recipient in &unsent {
            self.handle_custom_send(*recipient, email_id.eid).await?;
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        db_write_raw!(self.state, move |cmds| {
            cmds.account_admin()
                .custom_email()
                .mark_custom_email_sending_completed(email_id)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(EmailError::MarkAsSentFailed)?;

        Ok(())
    }

    async fn handle_custom_send(
        &self,
        recipient: AccountIdInternal,
        email_id: i64,
    ) -> error_stack::Result<(), EmailError> {
        let Some(info) = self.get_custom_email_data(recipient, email_id).await? else {
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

        self.mark_custom_as_sent(recipient, email_id).await
    }

    async fn send_draft_to_target(
        &self,
        recipient: AccountIdInternal,
        email_id: i64,
    ) -> error_stack::Result<(), EmailError> {
        let Some(info) = self.get_custom_email_data(recipient, email_id).await? else {
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

        Ok(())
    }

    /// If `Ok(None)` is returned the email sending is disabled for the
    /// provided `recipient`.
    async fn get_custom_email_data(
        &self,
        recipient: AccountIdInternal,
        message: i64,
    ) -> error_stack::Result<Option<EmailData>, EmailError> {
        let data = self
            .state
            .read()
            .account()
            .email_address_state(recipient)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        let email = if let Some(email) = data.email {
            if email.0.ends_with("@example.com") {
                return Ok(None);
            } else {
                email.0
            }
        } else {
            return Ok(None);
        };

        let email_id = CustomEmailId::new(message);

        let translations = self
            .state
            .read()
            .account_admin()
            .custom_email()
            .custom_email_translations(email_id)
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::GettingEmailDataFailed)?;

        let language = self
            .state
            .read()
            .common()
            .client_config()
            .client_language(recipient)
            .await
            .ok()
            .flatten();

        let translation = language
            .as_ref()
            .and_then(|lang| translations.iter().find(|t| t.locale == lang.as_str()))
            .or_else(|| translations.iter().find(|t| t.locale == "default"));

        let content = match translation {
            Some(t) => EmailContent {
                subject: t.subject.clone(),
                body: t.body.clone(),
                body_is_html: self
                    .state
                    .config()
                    .email_content()
                    .email_body_content_type_is_html(),
            },
            None => return Err(report!(EmailError::GettingEmailDataFailed)),
        };

        let email_data = EmailData {
            email_address: email,
            subject: content.subject,
            body: content.body,
            body_is_html: content.body_is_html,
        };

        Ok(Some(email_data))
    }

    async fn mark_custom_as_sent(
        &self,
        recipient: AccountIdInternal,
        message: i64,
    ) -> error_stack::Result<(), EmailError> {
        db_write_raw!(self.state, move |cmds| {
            cmds.account_admin()
                .custom_email()
                .mark_custom_email_sent(CustomEmailId::new(message), recipient)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(EmailError::MarkAsSentFailed)?;

        Ok(())
    }
}
