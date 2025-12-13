use std::sync::Arc;

use error_stack::{Result, report};
use test_mode_utils::client::TestError;
use tokio::sync::{Mutex, mpsc};
use tracing::error;

/// Trait for content-specific moderation handlers
pub trait ModerationHandler: Send + Sized {
    async fn handle(&mut self) -> Result<(), TestError>;

    /// Creates sender and receiver for a notification pipeline
    fn create_notification_channel(self) -> (NotificationSender, NotificationReceiver<Self>) {
        let (notify_sender, notify_receiver) = mpsc::unbounded_channel();
        let state = NotificationState::new(notify_sender);

        (
            NotificationSender {
                state: state.clone(),
            },
            NotificationReceiver {
                state,
                notify_receiver,
                handler: self,
            },
        )
    }
}

/// Shared state for notification pipeline
#[derive(Clone)]
struct NotificationState {
    pending: Arc<Mutex<bool>>,
    notify_sender: mpsc::UnboundedSender<()>,
}

impl NotificationState {
    fn new(notify_sender: mpsc::UnboundedSender<()>) -> Self {
        Self {
            pending: Arc::new(Mutex::new(false)),
            notify_sender,
        }
    }
}

/// Generic sender for notification pipeline
pub struct NotificationSender {
    state: NotificationState,
}

impl NotificationSender {
    /// Add notification to pending queue and signal if not already processing
    pub async fn notify(&self) {
        let mut pending = self.state.pending.lock().await;
        let should_notify = !*pending;
        *pending = true;
        drop(pending);

        if should_notify {
            let _ = self.state.notify_sender.send(());
        }
    }
}

/// Generic receiver for notification pipeline
pub struct NotificationReceiver<H: ModerationHandler> {
    state: NotificationState,
    notify_receiver: mpsc::UnboundedReceiver<()>,
    handler: H,
}

impl<H: ModerationHandler> NotificationReceiver<H> {
    pub async fn process_notifications_loop(&mut self) -> Result<(), TestError> {
        loop {
            match self.notify_receiver.recv().await {
                Some(()) => (),
                None => return Err(report!(TestError::AdminBotInternalError)),
            }

            let mut pending = self.state.pending.lock().await;
            if *pending {
                *pending = false;
                drop(pending);

                if let Err(e) = self.handler.handle().await {
                    error!("Moderation handler error: {:?}", e);
                }
            } else {
                drop(pending);
            }
        }
    }
}
