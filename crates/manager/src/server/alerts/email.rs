//! SMTP client for sending alert emails.

use std::str::FromStr;

use error_stack::ResultExt;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::{PoolConfig, authentication::Credentials},
};
use manager_config::file::EmailAlertsConfig;
use simple_backend_utils::{ContextExt, Result};

#[derive(thiserror::Error, Debug)]
pub enum AlertEmailError {
    #[error("Message building failed")]
    MessageBuildingFailed,
    #[error("Email sending failed")]
    SendingFailed,
    #[error("Email sending response not positive")]
    EmailSendingResponseNotPositive,
    #[error("SMTP transport creation failed")]
    TransportCreationFailed,
}

pub struct AlertEmailSender {
    sender: AsyncSmtpTransport<Tokio1Executor>,
    from_header: Mailbox,
    debug_logging: bool,
}

impl AlertEmailSender {
    pub fn new(config: &EmailAlertsConfig) -> Result<Self, AlertEmailError> {
        let transport_result = if config.use_starttls_instead_of_smtps {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server_address)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server_address)
        };

        let sender = transport_result
            .map(|builder| {
                builder
                    .credentials(Credentials::new(
                        config.username.clone(),
                        config.password.clone(),
                    ))
                    .pool_config(PoolConfig::new().max_size(1))
                    .build()
            })
            .map_err(|e| {
                AlertEmailError::TransportCreationFailed
                    .report()
                    .attach(format!("SMTP transport creation failed: {e}"))
            })?;

        let from_header = Mailbox::from_str(&config.email_from_header)
            .change_context(AlertEmailError::MessageBuildingFailed)
            .attach(format!(
                "Invalid email_from_header: {}",
                config.email_from_header
            ))?;

        Ok(Self {
            sender,
            from_header,
            debug_logging: config.debug_logging,
        })
    }

    pub async fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), AlertEmailError> {
        if self.debug_logging {
            tracing::info!("Sending alert email:\nTo: {to}\nSubject: {subject}\nBody: {body}");
        }

        self.send_inner(to, subject, body).await
    }

    async fn send_inner(&self, to: &str, subject: &str, body: &str) -> Result<(), AlertEmailError> {
        let message = Message::builder()
            .from(self.from_header.clone())
            .to(Mailbox::new(
                None,
                lettre::Address::from_str(to)
                    .change_context(AlertEmailError::MessageBuildingFailed)
                    .attach(format!("Invalid recipient address: {to}"))?,
            ))
            .subject(subject.to_string())
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .change_context(AlertEmailError::MessageBuildingFailed)?;

        let response = self
            .sender
            .send(message)
            .await
            .change_context(AlertEmailError::SendingFailed)?;

        if response.is_positive() {
            Ok(())
        } else {
            let response_message = response.message().collect::<Vec<_>>().join(" ");
            Err(AlertEmailError::EmailSendingResponseNotPositive
                .report()
                .attach(format!(
                    "SMTP response not positive, code: {}, message: {response_message}",
                    response.code()
                )))
        }
    }
}
