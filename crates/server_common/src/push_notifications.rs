use std::{future::Future, sync::Arc, time::Duration};

use config::Config;
use error_stack::{Result, ResultExt};
use fcm::{
    FcmClient,
    message::{AndroidConfig, AndroidMessagePriority, ApnsConfig, Message, Target},
};
use model::{
    AccountIdInternal, ClientLanguage, PendingNotificationFlags, PushNotificationStateInfoWithFlags,
};
use serde_json::json;
use simple_backend::ServerQuitWatcher;
use tokio::{
    sync::mpsc::{Receiver, Sender, error::TrySendError},
    task::JoinHandle,
    time::MissedTickBehavior,
};
use tracing::{error, warn};

use crate::push_notifications::logic::{FcmSendingLogic, UnusualAction};

mod logic;

const PUSH_NOTIFICATION_CHANNEL_BUFFER_SIZE: usize = 1024 * 1024;

#[derive(thiserror::Error, Debug)]
pub enum PushNotificationError {
    #[error("Creating FCM client failed")]
    CreateFcmClient,
    #[error("Reading or writing to database failed")]
    ReadOrWriteError,
    #[error("Reading notification sent status failed")]
    ReadingNotificationSentStatusFailed,
    #[error("Removing device token failed")]
    RemoveDeviceTokenFailed,
    #[error("Reading notification flags from cache failed")]
    ReadingNotificationFlagsFromCacheFailed,
    #[error("Notification visiblity check failed")]
    NotificationVisiblityCheckFailed,
    #[error("Saving pending notifications to database failed")]
    SaveToDatabaseFailed,
    #[error("Handling successful message sending action failed")]
    HandlingSuccessfulMessageSendingActionFailed,
    #[error("Get client language failed")]
    GetClientLanguageFailed,
}

pub struct PushNotificationManagerQuitHandle {
    task: JoinHandle<()>,
}

impl PushNotificationManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!(
                    "PushNotificationManagerQuitHandle quit failed. Error: {:?}",
                    e
                );
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
    fn get_push_notification_state_info(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Future<Output = Result<PushNotificationStateInfoWithFlags, PushNotificationError>> + Send;

    fn remove_device_token(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Future<Output = Result<(), PushNotificationError>> + Send;

    fn save_current_notification_flags_to_database_if_needed(
        &self,
    ) -> impl Future<Output = Result<(), PushNotificationError>> + Send;

    fn is_pending_notification_visible_notification(
        &self,
        account_id: AccountIdInternal,
        flags: PendingNotificationFlags,
    ) -> impl Future<Output = Result<bool, PushNotificationError>> + Send;

    fn client_language(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Future<Output = Result<ClientLanguage, PushNotificationError>> + Send;
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
    config: Arc<Config>,
    started_with_fcm_enabled: bool,
    fcm: Option<FcmClient>,
    receiver: PushNotificationReceiver,
    state: T,
}

impl<T: PushNotificationStateProvider + Send + 'static> PushNotificationManager<T> {
    pub async fn new_manager(
        config: Arc<Config>,
        quit_notification: ServerQuitWatcher,
        state: T,
        receiver: PushNotificationReceiver,
    ) -> PushNotificationManagerQuitHandle {
        let fcm = if let Some(config) = config.simple_backend().firebase_cloud_messaging_config() {
            // TODO(future): Make possible to use existing reqwest::Client
            //               with FcmClient.
            let fcm_result = FcmClient::builder()
                .service_account_key_json_path(&config.service_account_key_path)
                .token_cache_json_path(&config.token_cache_path)
                .fcm_request_timeout(Duration::from_secs(20))
                .build()
                .await;
            match fcm_result {
                Ok(client) => Some(client),
                Err(e) => {
                    error!("Creating FCM client failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let manager = PushNotificationManager {
            config,
            started_with_fcm_enabled: fcm.is_some(),
            fcm,
            receiver,
            state,
        };

        PushNotificationManagerQuitHandle {
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

        self.quit_logic().await;
    }

    pub async fn logic(&mut self) {
        let debug_logging = self
            .config
            .simple_backend()
            .firebase_cloud_messaging_config()
            .map(|v| v.debug_logging)
            .unwrap_or_default();
        let mut sending_logic = FcmSendingLogic::new(debug_logging);
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
            };

            match notification {
                Some(notification) => {
                    let result = self
                        .send_push_notification(notification, &mut sending_logic)
                        .await;
                    match result {
                        Ok(()) => (),
                        Err(e) => {
                            error!("Sending push notification failed: {:?}", e);
                        }
                    }
                }
                None => {
                    warn!("Push notification channel is broken");
                    break;
                }
            }
        }
    }

    pub async fn quit_logic(&mut self) {
        if self.started_with_fcm_enabled {
            // There might be unhandled or failed notifications, so save those
            // from cache to database.
            match self
                .state
                .save_current_notification_flags_to_database_if_needed()
                .await
            {
                Ok(()) => (),
                Err(e) => error!(
                    "Saving pending push notifications to database failed: {:?}",
                    e
                ),
            }
        }
    }

    pub async fn send_push_notification(
        &mut self,
        send_push_notification: SendPushNotification,
        sending_logic: &mut FcmSendingLogic,
    ) -> Result<(), PushNotificationError> {
        let fcm = if let Some(fcm) = &self.fcm {
            fcm
        } else {
            return Ok(());
        };

        let info = self
            .state
            .get_push_notification_state_info(send_push_notification.account_id)
            .await
            .change_context(PushNotificationError::ReadingNotificationSentStatusFailed)?;

        let (info, flags) = match info {
            PushNotificationStateInfoWithFlags::EmptyFlags => return Ok(()),
            PushNotificationStateInfoWithFlags::WithFlags { info, flags } => (info, flags),
        };

        let Some(token) = info.fcm_device_token else {
            return Ok(());
        };

        let is_visible = self
            .state
            .is_pending_notification_visible_notification(send_push_notification.account_id, flags)
            .await
            .change_context(PushNotificationError::NotificationVisiblityCheckFailed)?;

        let m = Message {
            // Use minimal notification data as this only triggers client
            // to download the notification.
            data: Some(json!({
                "n": "",
            })),
            target: Target::Token(token.into_string()),
            android: Some(AndroidConfig {
                priority: Some(if is_visible {
                    AndroidMessagePriority::High
                } else {
                    AndroidMessagePriority::Normal
                }),
                collapse_key: Some("0".to_string()),
                ..Default::default()
            }),
            apns: Some(ApnsConfig {
                headers: Some(json!({
                    // 5 is max priority for data notifications
                    "apns-priority": "5",
                    "apns-collapse-id": "0",
                })),
                payload: Some(json!({
                    "aps": {
                        "content-available": 1
                    }
                })),
                ..Default::default()
            }),
            webpush: None,
            fcm_options: None,
            notification: None,
        };

        match sending_logic.send_push_notification(m, fcm).await {
            Ok(()) => Ok(()),
            Err(action) => match action {
                UnusualAction::DisablePushNotificationSupport => {
                    self.fcm = None;
                    Ok(())
                }
                UnusualAction::RemoveDeviceToken => self
                    .state
                    .remove_device_token(send_push_notification.account_id)
                    .await
                    .change_context(PushNotificationError::RemoveDeviceTokenFailed),
            },
        }
    }
}
