use std::sync::Arc;

use error_stack::{Result, ResultExt};
use simple_backend_utils::ContextExt;
use tokio::sync::{Mutex, OwnedMutexGuard};

#[derive(thiserror::Error, Debug)]
pub enum InProgressCmdChannelError {
    #[error("Already locked")]
    AlreadyLocked,
    #[error("Command in progress")]
    CommandInProgress,
    #[error("Channel broken")]
    ChannelBroken,
}

#[derive(Debug, Clone)]
pub struct InProgressSender<T> {
    /// Is empty when previous message is handled.
    message_storage: Arc<Mutex<Option<T>>>,
    /// Notify receiver to handle the message.
    sender: tokio::sync::mpsc::Sender<()>,
}

impl<T> InProgressSender<T> {
    pub async fn send_message(&self, message: T) -> Result<(), InProgressCmdChannelError> {
        let mut current_message = self
            .message_storage
            .try_lock()
            .change_context(InProgressCmdChannelError::AlreadyLocked)?;
        if current_message.is_some() {
            return Err(InProgressCmdChannelError::CommandInProgress.report());
        } else {
            *current_message = Some(message);
        }

        drop(current_message);

        self.sender
            .send(())
            .await
            .change_context(InProgressCmdChannelError::ChannelBroken)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct InProgressReceiver<T> {
    /// Is empty when previous message is handled.
    message_storage: Arc<Mutex<Option<T>>>,
    /// New message available
    receiver: tokio::sync::mpsc::Receiver<()>,
}

impl<T> InProgressReceiver<T> {
    pub async fn is_new_message_available(&mut self) -> Result<(), InProgressCmdChannelError> {
        self.receiver
            .recv()
            .await
            .ok_or(InProgressCmdChannelError::ChannelBroken.report())?;
        Ok(())
    }

    pub async fn lock_message_container(&self) -> InProgressContainer<T> {
        let lock = self.message_storage.clone().lock_owned().await;

        InProgressContainer { in_progress: lock }
    }
}

/// Removes the current message once dropped.
pub struct InProgressContainer<T> {
    in_progress: OwnedMutexGuard<Option<T>>,
}

impl<T> InProgressContainer<T> {
    pub fn get_message(&self) -> Option<&T> {
        self.in_progress.as_ref()
    }
}

impl<T> Drop for InProgressContainer<T> {
    fn drop(&mut self) {
        *self.in_progress = None;
    }
}

/// Channel which allows to send only one message at a time and
/// wait for it to be handled.
pub struct InProgressChannel;

impl InProgressChannel {
    pub fn create<T>() -> (InProgressSender<T>, InProgressReceiver<T>) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        let mutex = Arc::new(Mutex::new(None));

        let sender = InProgressSender {
            message_storage: mutex.clone(),
            sender,
        };

        let receiver = InProgressReceiver {
            message_storage: mutex,
            receiver,
        };

        (sender, receiver)
    }
}
