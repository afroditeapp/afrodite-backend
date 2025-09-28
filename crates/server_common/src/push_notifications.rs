use std::{future::Future, sync::Arc, time::Duration};

use config::Config;
use error_stack::Result;
use model::{AccountIdInternal, PushNotificationSendingInfo};
use simple_backend::ServerQuitWatcher;
use tokio::{
    sync::mpsc::{Receiver, Sender, error::TrySendError},
    task::JoinHandle,
    time::MissedTickBehavior,
};
use tracing::{error, warn};

use crate::push_notifications::fcm::FcmManager;

mod fcm;

const PUSH_NOTIFICATION_CHANNEL_BUFFER_SIZE: usize = 1024 * 1024;

#[derive(thiserror::Error, Debug)]
pub enum PushNotificationError {
    #[error("Creating FCM client failed")]
    CreateFcmClient,
    #[error("Reading notification sent status failed")]
    ReadingNotificationSentStatusFailed,
    #[error("Removing device token failed")]
    RemoveDeviceTokenFailed,
    #[error("Get and reset push notifications failed")]
    GetAndResetPushNotificationsFailed,
    #[error("Saving pending notifications to database failed")]
    SaveToDatabaseFailed,
    #[error("Serializing error")]
    Serialize,
}

pub struct PushNotificationManagerQuitHandle {
    task: JoinHandle<()>,
}

impl PushNotificationManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("PushNotificationManagerQuitHandle quit failed. Error: {e:?}");
            }
        }
    }
}

/// New [PendingNotificationFlags] available in the cache.
#[derive(Debug, Clone, Copy)]
pub struct SendPushNotification {
    pub account_id: AccountIdInternal,
}

#[derive(Debug, Clone)]
pub struct PushNotificationSender {
    sender: Sender<SendPushNotification>,
    sender_low_priority: Sender<SendPushNotification>,
}

impl PushNotificationSender {
    pub fn send(&self, account_id: AccountIdInternal) {
        let notification = SendPushNotification { account_id };
        match self.sender.try_send(notification) {
            Ok(()) => (),
            Err(TrySendError::Closed(_)) => {
                error!("Sending push notification to internal channel failed: channel is broken");
            }
            Err(TrySendError::Full(_)) => {
                error!("Sending push notification to internal channel failed: channel is full");
            }
        }
    }

    pub fn send_low_priority(&self, account_id: AccountIdInternal) {
        let notification = SendPushNotification { account_id };
        match self.sender_low_priority.try_send(notification) {
            Ok(()) => (),
            Err(TrySendError::Closed(_)) => {
                error!(
                    "Sending low priority push notification to internal channel failed: channel is broken"
                );
            }
            Err(TrySendError::Full(_)) => {
                error!(
                    "Sending low priority push notification to internal channel failed: channel is full"
                );
            }
        }
    }
}

pub trait PushNotificationStateProvider {
    fn get_and_reset_push_notifications(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Future<Output = Result<PushNotificationSendingInfo, PushNotificationError>> + Send;

    fn remove_device_token(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Future<Output = Result<(), PushNotificationError>> + Send;

    fn save_current_notification_flags_to_database_if_needed(
        &self,
    ) -> impl Future<Output = Result<(), PushNotificationError>> + Send;
}

pub fn channel() -> (PushNotificationSender, PushNotificationReceiver) {
    let (sender, receiver) = tokio::sync::mpsc::channel(PUSH_NOTIFICATION_CHANNEL_BUFFER_SIZE);
    let (sender_low_priority, receiver_low_priority) =
        tokio::sync::mpsc::channel(PUSH_NOTIFICATION_CHANNEL_BUFFER_SIZE);
    let sender = PushNotificationSender {
        sender,
        sender_low_priority,
    };
    let receiver = PushNotificationReceiver {
        receiver,
        receiver_low_priority,
    };
    (sender, receiver)
}

#[derive(Debug)]
pub struct PushNotificationReceiver {
    receiver: Receiver<SendPushNotification>,
    receiver_low_priority: Receiver<SendPushNotification>,
}

pub struct PushNotificationManager<T> {
    fcm: FcmManager,
    receiver: PushNotificationReceiver,
    state: T,
}

impl<T: PushNotificationStateProvider + Send + Sync + 'static> PushNotificationManager<T> {
    pub async fn new_manager(
        config: Arc<Config>,
        quit_notification: ServerQuitWatcher,
        state: T,
        receiver: PushNotificationReceiver,
    ) -> PushNotificationManagerQuitHandle {
        let manager = PushNotificationManager {
            fcm: FcmManager::new(&config).await,
            receiver,
            state,
        };

        PushNotificationManagerQuitHandle {
            task: tokio::spawn(manager.run(quit_notification)),
        }
    }

    pub async fn run(mut self, mut quit_notification: ServerQuitWatcher) {
        self.logic(&mut quit_notification).await;
        // Make sure that quit started (closed channel also
        // breaks the logic loop, but that should not happen)
        let _ = quit_notification.recv().await;
        self.quit_logic().await;
    }

    pub async fn logic(&mut self, quit_notification: &mut ServerQuitWatcher) {
        let mut low_priority_notification_allowed = false;
        let mut low_priority_notification_interval =
            tokio::time::interval(Duration::from_millis(500));
        low_priority_notification_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            let notification = tokio::select! {
                notification = self.receiver.receiver.recv() => notification,
                notification = self.receiver.receiver_low_priority.recv(), if low_priority_notification_allowed => {
                    low_priority_notification_allowed = false;
                    low_priority_notification_interval.reset();
                    notification
                },
                _ = low_priority_notification_interval.tick(), if !low_priority_notification_allowed => {
                    low_priority_notification_allowed = true;
                    continue;
                }
                _ = quit_notification.recv() => return,
            };

            match notification {
                Some(notification) => match self.handle_notification(notification).await {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Sending push notification failed: {e:?}");
                    }
                },
                None => {
                    warn!("Push notification channel is broken");
                    return;
                }
            }
        }
    }

    pub async fn quit_logic(&mut self) {
        // There might be unhandled or failed notifications, so save those
        // from cache to database.
        match self
            .state
            .save_current_notification_flags_to_database_if_needed()
            .await
        {
            Ok(()) => (),
            Err(e) => error!("Saving pending push notifications to database failed: {e:?}"),
        }
    }

    pub async fn handle_notification(
        &mut self,
        send_push_notification: SendPushNotification,
    ) -> Result<(), PushNotificationError> {
        self.fcm
            .send_fcm_notification(send_push_notification, &self.state)
            .await
    }
}
