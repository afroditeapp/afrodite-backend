use error_stack::ResultExt;
use model::{AccountIdInternal, EmailMessages};
use server_data_account::read::GetReadCommandsAccount;
use server_state::S;
use simple_backend::email::{EmailData, EmailDataProvider, EmailError};
use server_api::{app::{GetConfig, ReadData, WriteData}, db_write_raw};
use server_data_account::write::GetWriteCommandsAccount;

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

        if data.email.is_none() {
            return Ok(None);
        }

        let email = if let Some(email) = data.email {
            if email.0.ends_with("example.com") {
                return Ok(None);
            }

            email.0
        } else {
            return Ok(None)
        };

        let email_content = self
            .state
            .config()
            .email_content()
            .ok_or(EmailError::GettingEmailDataFailed)
            .attach_printable("Email content not configured")?;

        let email_content = email_content.email
            .iter()
            .find(|e| e.message_type == message)
            .ok_or(EmailError::GettingEmailDataFailed)
            .attach_printable(format!("Email content for {:?} is not configured", message))?;

        let email_data = EmailData {
            email_address: email,
            subject: email_content.subject.clone(),
            body: email_content.body.clone(),
        };

        Ok(Some(email_data))
    }

    async fn mark_as_sent(
        &self,
        receiver: AccountIdInternal,
        message: EmailMessages,
    ) -> error_stack::Result<(), simple_backend::email::EmailError> {
        db_write_raw!(self.state, move |cmds| {
            cmds.account().email().mark_email_as_sent(receiver, message).await
        })
            .await
            .map_err(|e| e.into_report())
            .change_context(EmailError::MarkAsSentFailed)

    }
}
