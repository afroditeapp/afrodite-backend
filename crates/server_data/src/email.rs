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
const CUSTOM_EMAIL_CHANNEL_BUFFER_SIZE: usize = MIB_IN_BYTES;

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

pub struct CustomEmailMsg {
    /// CustomEmailId
    pub email_id: i64,
}

pub struct HighPriorityEmailMsg {
    pub recipient: model::AccountIdInternal,
    pub message: model::EmailMessages,
    pub result_sender: oneshot::Sender<Result<(), DataError>>,
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

    pub fn trigger_custom_email_sending(&self, email_id: i64) {
        let cmd = CustomEmailMsg { email_id };
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

    pub async fn send_high_priority(
        &self,
        recipient: model::AccountIdInternal,
        message: model::EmailMessages,
    ) -> error_stack::Result<(), DataError> {
        let (result_sender, result_receiver) = oneshot::channel();
        let cmd = HighPriorityEmailMsg {
            recipient,
            message,
            result_sender,
        };
        match self.high_priority_sender.try_send(cmd) {
            Ok(()) => (),
            Err(TrySendError::Closed(_)) => return Err(report!(DataError::EmailSendingFailed)),
            Err(TrySendError::Full(_)) => return Err(report!(DataError::EmailSendingFailed)),
        }

        match result_receiver.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) | Err(_) => Err(report!(DataError::EmailSendingFailed)),
        }
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
