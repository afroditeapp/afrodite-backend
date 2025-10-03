use std::{future::Future, sync::Arc, time::Duration};

use config::Config;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ClientType, PushNotificationSendingInfo};
use simple_backend::ServerQuitWatcher;
use simple_backend_utils::ContextExt;
use tokio::{
    sync::mpsc::{Receiver, Sender, error::TrySendError},
    task::JoinHandle,
    time::MissedTickBehavior,
};
use tracing::{error, warn};

use crate::push_notifications::{
    apns::{ApnsManager, ApnsManagerQuitHandle},
    fcm::{FcmManager, FcmManagerQuitHandle},
    web::{WebPushManager, WebPushManagerQuitHandle},
};

mod apns;
mod fcm;
mod web;

const PRIMARY_BUFFER_SIZE: usize = 1024 * 1024;
const SECONDARY_BUFFER_SIZE: usize = 1024 * 512;

#[derive(thiserror::Error, Debug)]
pub enum PushNotificationError {
    #[error("Creating FCM client failed")]
    CreateFcmClient,
    #[error("Creating APNs client failed")]
    CreateApnsClient,
    #[error("Creating web push notification client failed")]
    CreateWebPushClient,
    #[error("Reading notification sent status failed")]
    ReadingNotificationSentStatusFailed,
    #[error("Removing device token failed")]
    RemoveDeviceTokenFailed,
    #[error("Get and reset push notifications failed")]
    GetAndResetPushNotificationsFailed,
    #[error("Getting client type failed")]
    GetClientType,
    #[error("Client type not found")]
    ClientTypeNotFound,
    #[error("Saving pending notifications to database failed")]
    SaveToDatabaseFailed,
    #[error("Serializing error")]
    Serialize,
    #[error("Notification building error")]
    NotificationBuildingFailed,
    #[error("Notification routing failed")]
    NotificationRoutingFailed,
}

pub struct PushNotificationManagerQuitHandle {
    task: JoinHandle<()>,
}

impl PushNotificationManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("PushNotificationManager quit failed. Error: {e:?}");
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

    fn client_type(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Future<Output = Result<Option<ClientType>, PushNotificationError>> + Send;
}

pub fn channel() -> (PushNotificationSender, PushNotificationReceiver) {
    let (sender, receiver) = tokio::sync::mpsc::channel(PRIMARY_BUFFER_SIZE);
    let (sender_low_priority, receiver_low_priority) =
        tokio::sync::mpsc::channel(PRIMARY_BUFFER_SIZE);
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
    fcm_sender: Sender<SendPushNotification>,
    apns_sender: Sender<SendPushNotification>,
    web_sender: Sender<SendPushNotification>,
    fcm_quit_handle: FcmManagerQuitHandle,
    apns_quit_handle: ApnsManagerQuitHandle,
    web_quit_handle: WebPushManagerQuitHandle,
    receiver: PushNotificationReceiver,
    state: T,
}

impl<T: PushNotificationStateProvider + Clone + Send + Sync + 'static> PushNotificationManager<T> {
    pub async fn new_manager(
        config: Arc<Config>,
        quit_notification: ServerQuitWatcher,
        state: T,
        receiver: PushNotificationReceiver,
    ) -> PushNotificationManagerQuitHandle {
        let (fcm_sender, fcm_receiver) = tokio::sync::mpsc::channel(SECONDARY_BUFFER_SIZE);
        let fcm_quit_handle = FcmManager::new_manager(
            &config,
            fcm_receiver,
            state.clone(),
            quit_notification.resubscribe(),
        )
        .await;

        let (apns_sender, apns_receiver) = tokio::sync::mpsc::channel(SECONDARY_BUFFER_SIZE);
        let apns_quit_handle = ApnsManager::new_manager(
            &config,
            apns_receiver,
            state.clone(),
            quit_notification.resubscribe(),
        )
        .await;

        let (web_sender, web_receiver) = tokio::sync::mpsc::channel(SECONDARY_BUFFER_SIZE);
        let web_quit_handle = WebPushManager::new_manager(
            &config,
            web_receiver,
            state.clone(),
            quit_notification.resubscribe(),
        )
        .await;

        let manager = PushNotificationManager {
            fcm_sender,
            apns_sender,
            web_sender,
            fcm_quit_handle,
            apns_quit_handle,
            web_quit_handle,
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

    pub async fn quit_logic(self) {
        self.fcm_quit_handle.wait_quit().await;
        self.apns_quit_handle.wait_quit().await;
        self.web_quit_handle.wait_quit().await;

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
        let Some(client_type) = self
            .state
            .client_type(send_push_notification.account_id)
            .await?
        else {
            return Err(PushNotificationError::ClientTypeNotFound.report());
        };

        match client_type {
            ClientType::Android => self
                .fcm_sender
                .send(send_push_notification)
                .await
                .change_context(PushNotificationError::NotificationRoutingFailed),
            ClientType::Ios => self
                .apns_sender
                .send(send_push_notification)
                .await
                .change_context(PushNotificationError::NotificationRoutingFailed),
            ClientType::Web => self
                .web_sender
                .send(send_push_notification)
                .await
                .change_context(PushNotificationError::NotificationRoutingFailed),
        }
    }
}
