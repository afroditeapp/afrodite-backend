use std::sync::Arc;

use error_stack::{Context, Report, Result, ResultExt};
use tokio::sync::{oneshot, Mutex, OwnedMutexGuard};

/// Sender only used for quit request message sending.
pub type QuitSender = oneshot::Sender<()>;

/// Receiver only used for quit request message receiving.
pub type QuitReceiver = oneshot::Receiver<()>;

pub trait ContextExt: Context + Sized {
    #[track_caller]
    fn report(self) -> Report<Self> {
        error_stack::report!(self)
    }
}

impl<E: Context + Sized> ContextExt for E {}

pub trait ErrorConversion: ResultExt + Sized {
    type Err: Context;
    const ERROR: Self::Err;

    /// Change error context and add additional info about error.
    #[track_caller]
    fn with_info<I: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static>(
        self,
        info: I,
    ) -> Result<<Self as ResultExt>::Ok, Self::Err> {
        self.change_context(Self::ERROR).attach_printable(info)
    }

    /// Change error context and add additional info about error. Sets
    /// additional info about error lazily.
    #[track_caller]
    fn with_info_lazy<
        F: FnOnce() -> I,
        I: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    >(
        self,
        info: F,
    ) -> Result<<Self as ResultExt>::Ok, Self::Err> {
        self.change_context(Self::ERROR).attach_printable_lazy(info)
    }
}

pub type ErrorContainer<E> = Option<Report<E>>;

pub trait AppendErr: Sized {
    type E: Context;

    fn append(&mut self, e: Report<Self::E>);
    fn into_result(self) -> Result<(), Self::E>;
}

pub trait AppendErrorTo<Err>: Sized {
    fn append_to_and_ignore(self, container: &mut ErrorContainer<Err>);
    fn append_to_and_return_container(self, container: &mut ErrorContainer<Err>)
        -> Result<(), Err>;
}

impl<Ok, Err: Context> AppendErrorTo<Err> for Result<Ok, Err>
where
    ErrorContainer<Err>: AppendErr<E = Err>,
{
    fn append_to_and_ignore(self, container: &mut ErrorContainer<Err>) {
        if let Err(e) = self {
            container.append(e)
        }
    }

    fn append_to_and_return_container(
        self,
        container: &mut ErrorContainer<Err>,
    ) -> Result<(), Err> {
        if let Err(e) = self {
            container.append(e);
            container.take().into_result()
        } else {
            Ok(())
        }
    }
}

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
