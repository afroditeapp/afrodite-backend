
use std::{sync::Arc, time::Duration};

use fcm::{message::{Message, Notification, Target}, FcmClient, FcmResponseError, RecomendedAction, RecomendedWaitTime};

use crate::{app::ReadData, result::{Result, WrappedResultExt}};
use model::{AccountId, AccountIdInternal, FcmDeviceToken, NotificationEvent, PendingNotificationFlags};
use serde_json::{error, json};
use simple_backend::{app::SimpleBackendAppState, ServerQuitWatcher};
use simple_backend_config::SimpleBackendConfig;
use tokio::{sync::mpsc::{error::TrySendError, Receiver, Sender}, task::JoinHandle};
use tracing::{error, info, warn};

use crate::app::{AppState, WriteData};

const PUSH_NOTIFICATION_CHANNEL_BUFFER_SIZE: usize = 1024 * 1024;

#[derive(thiserror::Error, Debug)]
pub enum PushNotificationError {
    #[error("Creating FCM client failed")]
    CreateFcmClient,
    #[error("Reading notification sent status failed")]
    ReadingNotificationSentStatusFailed,
    #[error("Removing device token failed")]
    RemoveDeviceTokenFailed,
    #[error("Setting push notification sent flag failed")]
    SettingPushNotificationSentFlagFailed,
}

pub struct PushNotificationManagerQuitHandle {
    task: JoinHandle<()>,
}

impl PushNotificationManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("PushNotificationManagerQuitHandle quit failed. Error: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SendPushNotification {
    pub account_id: AccountIdInternal,
    pub event: NotificationEvent,
}

#[derive(Debug, Clone)]
pub struct PushNotificationSender {
    sender: Sender<SendPushNotification>,
}

impl PushNotificationSender {
    pub fn send(
        &self,
        account_id: AccountIdInternal,
        event: NotificationEvent,
    ) {
        let notification = SendPushNotification {
            account_id,
            event,
        };
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
}

#[derive(Debug)]
pub struct PushNotificationReceiver {
    receiver: Receiver<SendPushNotification>,
}

pub struct PushNotificationManager {
    fcm: Option<FcmClient>,
    receiver: PushNotificationReceiver,
    state: SimpleBackendAppState<AppState>,
    current_push_notification_in_sending: Option<SendPushNotification>,
}

impl PushNotificationManager {
    pub fn channel() -> (PushNotificationSender, PushNotificationReceiver) {
        let (sender, receiver) = tokio::sync::mpsc::channel(PUSH_NOTIFICATION_CHANNEL_BUFFER_SIZE);
        let sender = PushNotificationSender { sender };
        let receiver = PushNotificationReceiver { receiver };
        (sender, receiver)
    }

    pub async fn new_manager(
        config: &SimpleBackendConfig,
        quit_notification: ServerQuitWatcher,
        state: SimpleBackendAppState<AppState>,
        receiver: PushNotificationReceiver,
    ) -> PushNotificationManagerQuitHandle {
        let fcm = if let Some(config) = config.firebase_cloud_messaging_config() {
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
            fcm,
            receiver,
            state,
            current_push_notification_in_sending: None,
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
        let mut sending_logic = FcmSendingLogic::new();
        loop {
            let notification = self.receiver.receiver.recv().await;
            match notification {
                Some(notification) => {
                    self.current_push_notification_in_sending = Some(notification);
                    let result = self.send_push_notification(notification, &mut sending_logic).await;
                    if let Err(e) = result {
                        error!("Sending push notification failed: {:?}", e);
                        // TODO: Save notification?
                    }
                    self.current_push_notification_in_sending = None;
                }
                None => {
                    warn!("Push notification channel is broken");
                    break;
                },
            }
        }
    }

    pub async fn quit_logic(&mut self) {
        // TODO(prod): Save the current notification in sending to DB
        // TODO(prod): Read channel and save all pending notifications to DB
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

        let info = self.state
            .write(move |cmds| async move {
                let flags: PendingNotificationFlags = send_push_notification.event.into();
                cmds
                    .chat()
                    .push_notifications()
                    .get_push_notification_state_info_and_add_notification_value(
                        send_push_notification.account_id,
                        flags.into(),
                    )
                    .await
            })
            .await
            .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)?;

        if info.fcm_notification_sent {
            return Ok(());
        }

        let token = match info.fcm_device_token {
            Some(token) => token,
            None => return Ok(()),
        };

        let message = Message {
            data: Some(json!({
                "check_notifications": "",
            })),
            target: Target::Token(token.into_string()),
            android: None,
            apns: None,
            webpush: None,
            fcm_options: None,
            notification: None,
        };

        match sending_logic.send_push_notification(message, fcm).await {
            Ok(()) => {
                self.state
                    .write(move |cmds| async move {
                        cmds
                            .chat()
                            .push_notifications()
                            .enable_push_notification_sent_flag(send_push_notification.account_id)
                            .await
                    })
                    .await
                    .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)?;
                Ok(())
            }
            Err(action) => {
                match action {
                    UnusualAction::DisablePushNotificationSupport => {
                        self.fcm = None;
                        Ok(())
                    }
                    UnusualAction::RemoveDeviceToken =>
                        self.state
                            .write(move |cmds| async move {
                                cmds.chat().push_notifications()
                                    .remove_device_token(send_push_notification.account_id)
                                    .await
                            })
                            .await
                            .change_context(PushNotificationError::RemoveDeviceTokenFailed)
                }
            }
        }
    }
}

pub struct FcmSendingLogic {
    initial_send_rate_limit_millis: u64,
    exponential_backoff: Option<Duration>,
    forced_wait_time: Option<Duration>,
}

impl FcmSendingLogic {
    pub fn new() -> Self {
        Self {
            initial_send_rate_limit_millis: 1,
            exponential_backoff: None,
            forced_wait_time: None,
        }
    }

    pub async fn send_push_notification(
        &mut self,
        message: Message,
        fcm: &FcmClient,
    ) -> std::result::Result<(), UnusualAction> {
        self.exponential_backoff = None;
        self.forced_wait_time = None;

        loop {
            match self.retry_sending(&message, fcm).await {
                NextAction::NextMessage => return Ok(()),
                NextAction::UnusualAction(action) => return Err(action),
                NextAction::Retry => (),
            }
        }
    }

    async fn retry_sending(
        &mut self,
        message: &Message,
        fcm: &FcmClient,
    ) -> NextAction {
        match (self.forced_wait_time.take(), self.exponential_backoff) {
            (None, None) =>
                // First time trying to send this message.
                // Basic rate limiting might be good, so wait some time.
                tokio::time::sleep(Duration::from_millis(self.initial_send_rate_limit_millis)).await,
            (Some(forced_wait_time), _) =>
                tokio::time::sleep(forced_wait_time).await,
            (_, Some(exponential_backoff)) => {
                // TODO: Add some jitter time?
                let next_exponential_backoff =
                    exponential_backoff.as_millis() * exponential_backoff.as_millis();
                tokio::time::sleep(exponential_backoff).await;
                self.exponential_backoff = Some(Duration::from_millis(next_exponential_backoff as u64));
            }
        }

        match fcm.send(message).await {
            Ok(response) => {
                let action = response.recommended_error_handling_action();
                if let Some(action) = &action {
                    error!("FCM error detected, response: {:#?}, action: {:#?}", response, action);
                }
                match action {
                    None => {
                        // TODO(prod): Remove logging
                        info!("FCM send successful");
                        NextAction::NextMessage // No errors
                    }
                    Some(
                        RecomendedAction::CheckIosAndWebCredentials |
                        RecomendedAction::CheckSenderIdEquality |
                        RecomendedAction::FixMessageContent
                    ) => NextAction::UnusualAction(UnusualAction::DisablePushNotificationSupport),
                    Some(RecomendedAction::RemoveFcmAppToken) =>
                        NextAction::UnusualAction(UnusualAction::RemoveDeviceToken),
                    Some(RecomendedAction::ReduceMessageRateAndRetry(wait_time)) => {
                        self.initial_send_rate_limit_millis *= 2;
                        self.handle_recommended_wait_time(wait_time);
                        NextAction::Retry
                    },
                    Some(RecomendedAction::Retry(wait_time)) => {
                        self.handle_recommended_wait_time(wait_time);
                        NextAction::Retry
                    },
                    Some(RecomendedAction::HandleUnknownError) => {
                        // Just set forced wait time and hope for the best...
                        self.forced_wait_time = Some(Duration::from_secs(60));
                        NextAction::Retry
                    }
                }
            }
            Err(e) => {
                error!("FCM send failed: {:?}", e);
                if e.is_access_token_missing_even_if_server_requests_completed() {
                    NextAction::UnusualAction(UnusualAction::DisablePushNotificationSupport)
                } else {
                    // Just set forced wait time and hope for the best...
                    self.forced_wait_time = Some(Duration::from_secs(60));
                    NextAction::Retry
                }
            }
        }
    }

    fn handle_recommended_wait_time(&mut self, recommendation: RecomendedWaitTime) {
        match recommendation {
            RecomendedWaitTime::InitialWaitTime(wait_time) =>
                if self.exponential_backoff.is_none() {
                    // Set initial time for exponential back-off
                    self.exponential_backoff = Some(wait_time);
                }
            RecomendedWaitTime::SpecificWaitTime(retry_after) =>
                self.forced_wait_time = Some(retry_after.wait_time()),
        }
    }
}

impl Default for FcmSendingLogic {
    fn default() -> Self {
        Self::new()
    }
}

enum NextAction {
    UnusualAction(UnusualAction),
    NextMessage,
    Retry,
}

pub enum UnusualAction {
    DisablePushNotificationSupport,
    RemoveDeviceToken,
}
