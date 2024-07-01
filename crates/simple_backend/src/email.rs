



use std::{future::Future, num::NonZeroU32, str::FromStr, time::{Duration, Instant}};

use error_stack::{Result, ResultExt};
use lettre::{message::Mailbox, transport::smtp::{authentication::Credentials, PoolConfig}, Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use simple_backend_config::{file::EmailSendingConfig, SimpleBackendConfig};
use simple_backend_utils::ContextExt;
use tokio::{
    sync::mpsc::{error::TrySendError, Receiver, Sender},
    task::JoinHandle,
};
use tracing::{error, warn};

use crate::ServerQuitWatcher;

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
}

pub struct EmailManagerQuitHandle {
    task: JoinHandle<()>,
}

impl EmailManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!(
                    "EmailManagerQuitHandle quit failed. Error: {:?}",
                    e
                );
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

impl <R, M> EmailSender<R, M> {
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
}

pub trait EmailDataProvider<R, M> {
    /// If `Ok(None)` is returned the email sending is disabled for the
    /// provided `receiver`.
    fn get_email_data(
        &self,
        receiver: R,
        message: M,
    ) -> impl Future<Output = Result<Option<EmailData>, EmailError>> + Send;
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
}

pub struct EmailManager<T, R, M> {
    email_sender: Option<EmailSenderData>,
    receiver: EmailReceiver<R, M>,
    state: T,
}

impl<T: EmailDataProvider<R, M> + Send + 'static, R: Send + 'static, M: Send + 'static> EmailManager<T, R, M> {
    pub fn new_manager(
        config: &SimpleBackendConfig,
        quit_notification: ServerQuitWatcher,
        state: T,
        receiver: EmailReceiver<R, M>,
    ) -> EmailManagerQuitHandle {
        let email_sender = if let Some(config) = config.email_sending() {
            let email_sender = if config.use_starttls_instead_of_smtps {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server_address)
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server_address)
            }
            .map(|builder| {
                builder
                    .credentials(
                        Credentials::new(
                            config.username.clone(),
                            config.password.clone()
                        )
                    )
                    .pool_config(
                        PoolConfig::new()
                            .max_size(1)
                    )
                    .build()
            });

            match email_sender {
                Ok(email_sender) => Some(EmailSenderData {
                    sender: email_sender,
                    config: config.clone(),
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
        tokio::select! {
            _ = quit_notification.recv() => (),
            _ = self.logic() => (),
        }

        // Make sure that quit started (closed channel also
        // breaks the logic loop, but that should not happen)
        let _ = quit_notification.recv().await;
    }

    pub async fn logic(&mut self) {
        let mut sending_logic = EmailSendingLogic::new();
        loop {
            let send_cmd = self.receiver.receiver.recv().await;
            match send_cmd {
                Some(send_cmd) => {
                    let result = self
                        .send_email(send_cmd, &mut sending_logic)
                        .await;
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
            .get_email_data(
                send_cmd.receiver,
                send_cmd.message,
            )
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

        let message = Message::builder()
            .from(email_sender.config.email_from_header.0.clone())
            .to(Mailbox::new(None, address))
            .subject(info.subject)
            .body(info.body)
            .change_context(EmailError::MessageBuildingFailed)?;

        sending_logic.send_email(message, email_sender).await
    }
}

pub struct EmailSendingLogic {
    send_count_per_minute: SendCounter,
    send_count_per_day: SendCounter,
}

impl EmailSendingLogic {
    pub fn new() -> Self {
        Self {
            send_count_per_day: SendCounter::new(Duration::from_secs(60*60*24)),
            send_count_per_minute: SendCounter::new(Duration::from_secs(60)),
        }
    }

    async fn send_email(
        &mut self,
        message: Message,
        sender: &EmailSenderData,
    ) -> Result<(), EmailError> {
        self.send_count_per_minute.wait_until_allowed(sender.config.send_limit_per_minute).await;
        self.send_count_per_day.wait_until_allowed(sender.config.send_limit_per_day).await;

        self.send_count_per_minute.increment(sender.config.send_limit_per_minute);
        self.send_count_per_day.increment(sender.config.send_limit_per_day);

        let response = sender.sender
            .send(message)
            .await
            .change_context(EmailError::SendingFailed)?;

        if response.is_positive() {
            Ok(())
        } else {
            let response_message = response.message().collect::<Vec<_>>().join(" ");
            let error = format!("SMTP response not positive, code: {}, message: {}", response.code(), response_message);
            Err(
                EmailError::EmailSendingResponseNotPositive.report()
                    .attach_printable(error)
            )
        }
    }
}

impl Default for EmailSendingLogic {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SendCounter {
    count: u32,
    last_reset: Instant,
    counter_duration: Duration,
}

impl SendCounter {
    pub fn new(counter_duration: Duration) -> Self {
        Self {
            count: 0,
            last_reset: Instant::now(),
            counter_duration,
        }
    }

    pub async fn wait_until_allowed(&mut self, limit: Option<NonZeroU32>) {
        if let Some(limit) = limit {
            if self.count >= limit.get() {
                // Limit reached
                self.wait_until_next_reset().await;
                self.count = 0;
                self.last_reset = Instant::now();
            }
        }
    }

    pub fn increment(&mut self, limit: Option<NonZeroU32>) {
        if limit.is_some() {
            self.count += 1;
        }
    }

    async fn wait_until_next_reset(&self) {
        let time_since_reset = Instant::now().duration_since(self.last_reset);
        if let Some(remaining_time) = self.counter_duration.checked_sub(time_since_reset) {
            tokio::time::sleep(remaining_time).await
        }
    }
}
