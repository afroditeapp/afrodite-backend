use std::{num::NonZeroU32, str::FromStr, time::Duration};

use data::EmailLimitStateStorage;
use error_stack::ResultExt;
use lettre::{
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::{PoolConfig, authentication::Credentials},
};
use simple_backend_config::{SimpleBackendConfig, file::EmailSendingConfig};
use simple_backend_model::UnixTime;
use simple_backend_utils::{ContextExt, Result};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::email::data::Counter;

mod data;

#[derive(thiserror::Error, Debug)]
pub enum EmailError {
    #[error("Email sending failed")]
    SendingFailed,
    #[error("Email sending response not positive")]
    EmailSendingResponseNotPositive,
    #[error("Getting email data failed")]
    GettingEmailDataFailed,
    #[error("Account email address parsing failed")]
    AccountEmailAddressParsingFailed,
    #[error("Message building failed")]
    MessageBuildingFailed,
    #[error("Mark as sent failed")]
    MarkAsSentFailed,

    // State saving and loading
    #[error("Loading saved state failed")]
    LoadSavedStateFailed,
    #[error("Removing saved state failed")]
    RemovingSavedStateFailed,
    #[error("Saving state failed")]
    SavingStateFailed,
}

pub struct SmtpClient {
    sending_logic: Option<Mutex<EmailSendingLogic>>,
}

impl SmtpClient {
    pub async fn new(simple_backend_config: &SimpleBackendConfig) -> Self {
        let data = if let Some(config) = simple_backend_config.email_sending() {
            let transport = if config.use_starttls_instead_of_smtps {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server_address)
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server_address)
            }
            .map(|builder| {
                builder
                    .credentials(Credentials::new(
                        config.username.clone(),
                        config.password.clone(),
                    ))
                    .pool_config(PoolConfig::new().max_size(1))
                    .build()
            });

            match transport {
                Ok(sender) => Some((sender, config.clone())),
                Err(e) => {
                    error!("Email sender creating failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let client = Self {
            sending_logic: data
                .map(|(sender, config)| Mutex::new(EmailSendingLogic::new(sender, config))),
        };

        client.load_state(simple_backend_config).await;

        client
    }

    async fn load_state(&self, config: &SimpleBackendConfig) {
        let state = match EmailLimitStateStorage::load_and_remove(config).await {
            Ok(state) => state,
            Err(e) => {
                error!("Loading email state failed, error: {:?}", e);
                EmailLimitStateStorage::default()
            }
        };

        let mut logic = match &self.sending_logic {
            Some(l) => l.lock().await,
            None => return,
        };
        logic
            .send_count_per_minute
            .load(state.emails_sent_per_minute);
        logic.send_count_per_day.load(state.emails_sent_per_day);
    }

    pub async fn save_state(&self, config: &SimpleBackendConfig) {
        let logic = match &self.sending_logic {
            Some(l) => l.lock().await,
            None => return,
        };
        let state = EmailLimitStateStorage {
            emails_sent_per_minute: logic.send_count_per_minute.to_count(),
            emails_sent_per_day: logic.send_count_per_day.to_count(),
        };
        drop(logic);

        match state.save(config).await {
            Ok(()) => (),
            Err(e) => {
                error!("Email sender state saving failed, error: {:?}", e);
            }
        }
    }

    /// Might block until email sending is possible
    pub async fn send(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        body_is_html: bool,
    ) -> Result<(), EmailError> {
        let mut sender = match &self.sending_logic {
            Some(s) => s.lock().await,
            None => return Ok(()),
        };
        sender.send(to, subject, body, body_is_html).await
    }
}

struct EmailSendingLogic {
    sender: AsyncSmtpTransport<Tokio1Executor>,
    config: EmailSendingConfig,
    send_count_per_minute: SendCounter,
    send_count_per_day: SendCounter,
}

impl EmailSendingLogic {
    fn new(sender: AsyncSmtpTransport<Tokio1Executor>, config: EmailSendingConfig) -> Self {
        Self {
            sender,
            config,
            send_count_per_day: SendCounter::new(Duration::from_secs(60 * 60 * 24)),
            send_count_per_minute: SendCounter::new(Duration::from_secs(60)),
        }
    }

    pub async fn send(
        &mut self,
        to: &str,
        subject: &str,
        body: &str,
        body_is_html: bool,
    ) -> Result<(), EmailError> {
        let address =
            Address::from_str(to).change_context(EmailError::AccountEmailAddressParsingFailed)?;

        let content_type = if body_is_html {
            ContentType::TEXT_HTML
        } else {
            ContentType::TEXT_PLAIN
        };

        if self.config.debug_logging {
            info!(
                "Sending email:\nTo: {}\nSubject: {}\nBody: {}",
                address, subject, body
            );
        }

        let message = Message::builder()
            .from(self.config.email_from_header.0.clone())
            .to(Mailbox::new(None, address))
            .subject(subject.to_string())
            .header(content_type)
            .body(body.to_string())
            .change_context(EmailError::MessageBuildingFailed)?;

        self.send_raw(message).await
    }

    async fn send_raw(&mut self, message: Message) -> Result<(), EmailError> {
        self.send_count_per_minute
            .wait_until_allowed(self.config.send_limit_per_minute)
            .await;
        self.send_count_per_day
            .wait_until_allowed(self.config.send_limit_per_day)
            .await;

        self.send_count_per_minute
            .increment(self.config.send_limit_per_minute);
        self.send_count_per_day
            .increment(self.config.send_limit_per_day);

        let response = self
            .sender
            .send(message)
            .await
            .change_context(EmailError::SendingFailed)?;

        if response.is_positive() {
            Ok(())
        } else {
            let response_message = response.message().collect::<Vec<_>>().join(" ");
            let error = format!(
                "SMTP response not positive, code: {}, message: {}",
                response.code(),
                response_message
            );
            Err(EmailError::EmailSendingResponseNotPositive
                .report()
                .attach(error))
        }
    }
}

struct SendCounter {
    value: u32,
    previous_reset: UnixTime,
    counter_duration: Duration,
}

impl SendCounter {
    fn new(counter_duration: Duration) -> Self {
        Self {
            value: 0,
            previous_reset: UnixTime::current_time(),
            counter_duration,
        }
    }

    fn load(&mut self, counter: Counter) {
        self.value = counter.value;
        self.previous_reset = counter.previous_reset;
    }

    fn to_count(&self) -> Counter {
        Counter {
            value: self.value,
            previous_reset: self.previous_reset,
        }
    }

    async fn wait_until_allowed(&mut self, limit: Option<NonZeroU32>) {
        if let Some(limit) = limit
            && self.value >= limit.get()
        {
            // Limit reached
            self.wait_until_next_reset().await;
            self.value = 0;
            self.previous_reset = UnixTime::current_time();
        }
    }

    fn increment(&mut self, limit: Option<NonZeroU32>) {
        if limit.is_some() {
            self.value += 1;
        }
    }

    async fn wait_until_next_reset(&self) {
        let seconds_since_reset =
            TryInto::<u64>::try_into(UnixTime::current_time().ut - self.previous_reset.ut)
                .unwrap_or(0);
        let time_since_reset = Duration::from_secs(seconds_since_reset);
        if let Some(remaining_time) = self.counter_duration.checked_sub(time_since_reset) {
            tokio::time::sleep(remaining_time).await
        }
    }
}
