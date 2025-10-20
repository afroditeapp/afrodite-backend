use std::{future::Future, num::NonZeroU32, str::FromStr, time::Duration};

use data::EmailLimitStateStorage;
use error_stack::{Result, ResultExt};
use lettre::{
    Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::{PoolConfig, authentication::Credentials},
};
use simple_backend_config::{SimpleBackendConfig, file::EmailSendingConfig};
use simple_backend_model::UnixTime;
use simple_backend_utils::ContextExt;
use tokio::{
    sync::mpsc::{Receiver, Sender, error::TrySendError},
    task::JoinHandle,
};
use tracing::{debug, error, warn};

use crate::{ServerQuitWatcher, email::data::Counter};

mod data;

const EMAIL_SENDING_CHANNEL_BUFFER_SIZE: usize = 1024 * 1024;

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

pub struct EmailManagerQuitHandle {
    task: JoinHandle<()>,
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

#[derive(Debug)]
pub struct SendEmail<R, M> {
    pub receiver: R,
    pub message: M,
}

#[derive(Debug, Clone)]
pub struct EmailSender<R, M> {
    sender: Sender<SendEmail<R, M>>,
}

impl<R, M> EmailSender<R, M> {
    pub fn send(&self, receiver: R, message: M) {
        let email_send_cmd = SendEmail { receiver, message };
        match self.sender.try_send(email_send_cmd) {
            Ok(()) => (),
            Err(TrySendError::Closed(_)) => {
                error!("Sending email to internal channel failed: channel is broken");
            }
            Err(TrySendError::Full(_)) => {
                error!("Sending email to internal channel failed: channel is full");
            }
        }
    }
}

pub struct EmailData {
    pub email_address: String,
    pub subject: String,
    pub body: String,
    pub body_is_html: bool,
}

pub trait EmailDataProvider<R, M> {
    /// If `Ok(None)` is returned the email sending is disabled for the
    /// provided `receiver`.
    fn get_email_data(
        &self,
        receiver: R,
        message: M,
    ) -> impl Future<Output = Result<Option<EmailData>, EmailError>> + Send;

    fn mark_as_sent(
        &self,
        receiver: R,
        message: M,
    ) -> impl Future<Output = Result<(), EmailError>> + Send;
}

pub fn channel<R, M>() -> (EmailSender<R, M>, EmailReceiver<R, M>) {
    let (sender, receiver) = tokio::sync::mpsc::channel(EMAIL_SENDING_CHANNEL_BUFFER_SIZE);
    let sender = EmailSender { sender };
    let receiver = EmailReceiver { receiver };
    (sender, receiver)
}

#[derive(Debug)]
pub struct EmailReceiver<R, M> {
    receiver: Receiver<SendEmail<R, M>>,
}

struct EmailSenderData {
    sender: AsyncSmtpTransport<Tokio1Executor>,
    config: EmailSendingConfig,
    simple_backend_config: SimpleBackendConfig,
    previous_state: EmailLimitStateStorage,
}

pub struct EmailManager<T, R, M> {
    email_sender: Option<EmailSenderData>,
    receiver: EmailReceiver<R, M>,
    state: T,
}

impl<
    T: EmailDataProvider<R, M> + Send + Sync + 'static,
    R: Clone + Send + 'static,
    M: Clone + Send + 'static,
> EmailManager<T, R, M>
{
    pub async fn new_manager(
        simple_backend_config: &SimpleBackendConfig,
        quit_notification: ServerQuitWatcher,
        state: T,
        receiver: EmailReceiver<R, M>,
    ) -> EmailManagerQuitHandle {
        let email_sender = if let Some(config) = simple_backend_config.email_sending() {
            let email_sender = if config.use_starttls_instead_of_smtps {
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

            let previous_state =
                match EmailLimitStateStorage::load_and_remove(simple_backend_config).await {
                    Ok(state) => state,
                    Err(e) => {
                        error!("Loading previous state failed, error: {:?}", e);
                        EmailLimitStateStorage::default()
                    }
                };

            match email_sender {
                Ok(email_sender) => Some(EmailSenderData {
                    sender: email_sender,
                    config: config.clone(),
                    simple_backend_config: simple_backend_config.clone(),
                    previous_state,
                }),
                Err(e) => {
                    error!("Email sender creating failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let manager = EmailManager {
            email_sender,
            receiver,
            state,
        };

        EmailManagerQuitHandle {
            task: tokio::spawn(manager.run(quit_notification)),
        }
    }

    pub async fn run(mut self, mut quit_notification: ServerQuitWatcher) {
        let mut sending_logic = EmailSendingLogic::new();
        if let Some(sender_data) = &self.email_sender {
            sending_logic
                .send_count_per_minute
                .load(sender_data.previous_state.emails_sent_per_minute);
            sending_logic
                .send_count_per_day
                .load(sender_data.previous_state.emails_sent_per_day);
        }

        tokio::select! {
            _ = quit_notification.recv() => (),
            _ = self.logic(&mut sending_logic) => (),
        }

        // Make sure that quit started (closed channel also
        // breaks the logic loop, but that should not happen)
        let _ = quit_notification.recv().await;

        self.before_quit(&sending_logic).await;
    }

    async fn before_quit(&self, sending_logic: &EmailSendingLogic) {
        let current_state = EmailLimitStateStorage {
            emails_sent_per_minute: sending_logic.send_count_per_minute.to_count(),
            emails_sent_per_day: sending_logic.send_count_per_day.to_count(),
        };
        if let Some(sender_data) = &self.email_sender {
            match current_state.save(&sender_data.simple_backend_config).await {
                Ok(()) => (),
                Err(e) => {
                    error!("Email sender state saving failed, error: {:?}", e);
                }
            }
        }
    }

    pub async fn logic(&mut self, sending_logic: &mut EmailSendingLogic) {
        loop {
            let send_cmd = self.receiver.receiver.recv().await;
            match send_cmd {
                Some(send_cmd) => {
                    let result = self.send_email(send_cmd, sending_logic).await;
                    match result {
                        Ok(()) => (),
                        Err(e) => {
                            error!("Email sending failed: {:?}", e);
                        }
                    }
                }
                None => {
                    warn!("Email channel is broken");
                    break;
                }
            }
        }
    }

    pub async fn send_email(
        &mut self,
        send_cmd: SendEmail<R, M>,
        sending_logic: &mut EmailSendingLogic,
    ) -> Result<(), EmailError> {
        let email_sender = if let Some(email_sender) = &self.email_sender {
            email_sender
        } else {
            return Ok(());
        };

        let info = self
            .state
            .get_email_data(send_cmd.receiver.clone(), send_cmd.message.clone())
            .await
            .change_context(EmailError::GettingEmailDataFailed)?;

        let info = if let Some(info) = info {
            info
        } else {
            // Email disabled for the email receiver
            return Ok(());
        };

        let address = Address::from_str(&info.email_address)
            .change_context(EmailError::AccountEmailAddressParsingFailed)?;

        let content_type = if info.body_is_html {
            ContentType::TEXT_HTML
        } else {
            ContentType::TEXT_PLAIN
        };

        let message = Message::builder()
            .from(email_sender.config.email_from_header.0.clone())
            .to(Mailbox::new(None, address))
            .subject(info.subject)
            .header(content_type)
            .body(info.body)
            .change_context(EmailError::MessageBuildingFailed)?;

        match sending_logic.send_email(message, email_sender).await {
            Ok(()) => {
                self.state
                    .mark_as_sent(send_cmd.receiver, send_cmd.message)
                    .await
            }
            e => e,
        }
    }
}

pub struct EmailSendingLogic {
    send_count_per_minute: SendCounter,
    send_count_per_day: SendCounter,
}

impl EmailSendingLogic {
    pub fn new() -> Self {
        Self {
            send_count_per_day: SendCounter::new(Duration::from_secs(60 * 60 * 24)),
            send_count_per_minute: SendCounter::new(Duration::from_secs(60)),
        }
    }

    async fn send_email(
        &mut self,
        message: Message,
        sender: &EmailSenderData,
    ) -> Result<(), EmailError> {
        self.send_count_per_minute
            .wait_until_allowed(sender.config.send_limit_per_minute)
            .await;
        self.send_count_per_day
            .wait_until_allowed(sender.config.send_limit_per_day)
            .await;

        self.send_count_per_minute
            .increment(sender.config.send_limit_per_minute);
        self.send_count_per_day
            .increment(sender.config.send_limit_per_day);

        if sender.config.debug_logging {
            debug!("Sending email: {:?}", message);
        }

        let response = sender
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
                .attach_printable(error))
        }
    }
}

impl Default for EmailSendingLogic {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SendCounter {
    value: u32,
    previous_reset: UnixTime,
    counter_duration: Duration,
}

impl SendCounter {
    pub fn new(counter_duration: Duration) -> Self {
        Self {
            value: 0,
            previous_reset: UnixTime::current_time(),
            counter_duration,
        }
    }

    pub fn load(&mut self, counter: Counter) {
        self.value = counter.value;
        self.previous_reset = counter.previous_reset;
    }

    pub fn to_count(&self) -> Counter {
        Counter {
            value: self.value,
            previous_reset: self.previous_reset,
        }
    }

    pub async fn wait_until_allowed(&mut self, limit: Option<NonZeroU32>) {
        if let Some(limit) = limit {
            if self.value >= limit.get() {
                // Limit reached
                self.wait_until_next_reset().await;
                self.value = 0;
                self.previous_reset = UnixTime::current_time();
            }
        }
    }

    pub fn increment(&mut self, limit: Option<NonZeroU32>) {
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
