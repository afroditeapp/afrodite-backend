use error_stack::report;
pub use simple_backend::email::EmailError;
use simple_backend_utils::consts::MIB_IN_BYTES;
use tokio::sync::{
    mpsc::{Receiver, Sender, error::TrySendError},
    oneshot,
};
use tracing::error;

use crate::DataError;

const EMAIL_CHANNEL_BUFFER_SIZE: usize = MIB_IN_BYTES;
const EMAIL_HIGH_PRIORITY_CHANNEL_BUFFER_SIZE: usize = MIB_IN_BYTES;
const CUSTOM_EMAIL_CHANNEL_BUFFER_SIZE: usize = 16;

pub struct EmailData {
    pub email_address: String,
    pub subject: String,
    pub body: String,
    pub body_is_html: bool,
}

pub struct NormalEmailMsg {
    pub recipient: model::AccountIdInternal,
    pub message: model::EmailMessages,
}

pub enum CustomEmailMsg {
    /// Send to all accounts that haven't received it yet.
    SendToAll { email_id: i64 },
    /// Send a draft to one specific account, no DB persistence.
    SendDraft {
        email_id: i64,
        target_account_id: model::AccountIdInternal,
    },
}

pub enum HighPriorityEmailMsg {
    Normal {
        recipient: model::AccountIdInternal,
        message: model::EmailMessages,
        result_sender: oneshot::Sender<Result<(), DataError>>,
    },
    RegistrationToken {
        email: String,
        token: String,
        result_sender: oneshot::Sender<Result<(), DataError>>,
    },
}

/// A handle that can be awaited to get the result of sending a high priority email.
/// Created by [`EmailChannelSender::send_high_priority`].
pub struct EmailSendingHandle {
    result_receiver: oneshot::Receiver<Result<(), DataError>>,
}

impl EmailSendingHandle {
    /// Wait for the email sending result.
    pub async fn wait(self) -> Result<(), DataError> {
        match self.result_receiver.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) | Err(_) => Err(DataError::EmailSendingFailed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmailChannelSender {
    sender: Sender<NormalEmailMsg>,
    high_priority_sender: Sender<HighPriorityEmailMsg>,
    custom_sender: Sender<CustomEmailMsg>,
}

impl EmailChannelSender {
    pub fn send(&self, recipient: model::AccountIdInternal, message: model::EmailMessages) {
        let cmd = NormalEmailMsg { recipient, message };
        match self.sender.try_send(cmd) {
            Ok(()) => (),
            Err(TrySendError::Closed(_)) => {
                error!("Email channel send failed: channel broken");
            }
            Err(TrySendError::Full(_)) => {
                error!("Email channel send failed: channel full");
            }
        }
    }

    pub fn trigger_custom_email_sending(
        &self,
        email_id: i64,
        target_account_id: Option<model::AccountIdInternal>,
    ) {
        let cmd = match target_account_id {
            Some(target_account_id) => CustomEmailMsg::SendDraft {
                email_id,
                target_account_id,
            },
            None => CustomEmailMsg::SendToAll { email_id },
        };
        match self.custom_sender.try_send(cmd) {
            Ok(()) => (),
            Err(TrySendError::Closed(_)) => {
                error!("Custom email channel send failed: channel broken");
            }
            Err(TrySendError::Full(_)) => {
                error!("Custom email channel send failed: channel full");
            }
        }
    }

    pub fn send_high_priority(
        &self,
        recipient: model::AccountIdInternal,
        message: model::EmailMessages,
    ) -> error_stack::Result<EmailSendingHandle, DataError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let cmd = HighPriorityEmailMsg::Normal {
            recipient,
            message,
            result_sender,
        };
        match self.high_priority_sender.try_send(cmd) {
            Ok(()) => (),
            Err(TrySendError::Closed(_)) => return Err(report!(DataError::EmailSendingFailed)),
            Err(TrySendError::Full(_)) => return Err(report!(DataError::EmailSendingFailed)),
        }

        Ok(EmailSendingHandle { result_receiver })
    }

    pub fn send_registration_login_token(
        &self,
        email: String,
        token: String,
    ) -> error_stack::Result<EmailSendingHandle, DataError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let cmd = HighPriorityEmailMsg::RegistrationToken {
            email,
            token,
            result_sender,
        };
        match self.high_priority_sender.try_send(cmd) {
            Ok(()) => (),
            Err(TrySendError::Closed(_)) => return Err(report!(DataError::EmailSendingFailed)),
            Err(TrySendError::Full(_)) => return Err(report!(DataError::EmailSendingFailed)),
        }

        Ok(EmailSendingHandle { result_receiver })
    }
}

pub fn email_channel() -> (
    EmailChannelSender,
    Receiver<NormalEmailMsg>,
    Receiver<HighPriorityEmailMsg>,
    Receiver<CustomEmailMsg>,
) {
    let (sender, receiver) = tokio::sync::mpsc::channel(EMAIL_CHANNEL_BUFFER_SIZE);
    let (hp_sender, hp_receiver) =
        tokio::sync::mpsc::channel(EMAIL_HIGH_PRIORITY_CHANNEL_BUFFER_SIZE);
    let (custom_sender, custom_receiver) =
        tokio::sync::mpsc::channel(CUSTOM_EMAIL_CHANNEL_BUFFER_SIZE);
    (
        EmailChannelSender {
            sender,
            high_priority_sender: hp_sender,
            custom_sender,
        },
        receiver,
        hp_receiver,
        custom_receiver,
    )
}
