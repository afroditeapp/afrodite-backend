use std::{future::Future, time::Duration};

use error_stack::{Result, ResultExt};
use fcm::{
    FcmClient,
    message::Message,
    response::{RecomendedAction, RecomendedWaitTime},
};
use model::{
    AccountIdInternal, FcmDeviceToken, PendingMessageIdInternal, PendingNotificationFlags,
    PushNotificationStateInfoWithFlags,
};
use serde_json::Value;
use simple_backend::ServerQuitWatcher;
use simple_backend_config::SimpleBackendConfig;
use tokio::{
    sync::mpsc::{Receiver, Sender, error::TrySendError},
    task::JoinHandle,
    time::MissedTickBehavior,
};
use tracing::{error, info, warn};

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
    #[error("Removing specific notification flags from cache failed")]
    RemoveSpecificNotificationFlagsFromCacheFailed,
    #[error("Reading notification flags from cache failed")]
    ReadingNotificationFlagsFromCacheFailed,
    #[error("Creating push notifications failed")]
    CreatingPushNotificationsFailed,
    #[error("Saving pending notifications to database failed")]
    SaveToDatabaseFailed,
    #[error("Handling successful message sending action failed")]
    HandlingSuccessfulMessageSendingActionFailed,
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

pub enum SuccessfulSendingAction {
    MarkMessageNotificationSent { message: PendingMessageIdInternal },
}

pub struct PushNotification {
    /// FCM message which will be sent
    pub message: Option<Message>,
    /// Remove these flags from cache if Message is sent successfully
    pub flags: PendingNotificationFlags,
    pub successful_sending_action: Option<SuccessfulSendingAction>,
}

pub trait PushNotificationStateProvider {
    fn get_push_notification_state_info_and_add_notification_value(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Future<Output = Result<PushNotificationStateInfoWithFlags, PushNotificationError>> + Send;

    fn enable_push_notification_sent_flag(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Future<Output = Result<(), PushNotificationError>> + Send;

    fn remove_device_token(
        &self,
        account_id: AccountIdInternal,
    ) -> impl Future<Output = Result<(), PushNotificationError>> + Send;

    /// Avoid saving the cached notification to DB when server closes.
    fn remove_specific_notification_flags_from_cache(
        &self,
        account_id: AccountIdInternal,
        flags: PendingNotificationFlags,
    ) -> impl Future<Output = Result<(), PushNotificationError>> + Send;

    fn save_current_non_empty_notification_flags_from_cache_to_database(
        &self,
    ) -> impl Future<Output = Result<(), PushNotificationError>> + Send;

    fn convert_to_push_notifications(
        &self,
        account_id: AccountIdInternal,
        token: FcmDeviceToken,
        flags: PendingNotificationFlags,
    ) -> impl Future<Output = Result<Vec<PushNotification>, PushNotificationError>> + Send;

    fn handle_successful_message_sending_action(
        &self,
        action: SuccessfulSendingAction,
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
    started_with_fcm_enabled: bool,
    fcm: Option<FcmClient>,
    receiver: PushNotificationReceiver,
    state: T,
}

impl<T: PushNotificationStateProvider + Send + 'static> PushNotificationManager<T> {
    pub async fn new_manager(
        config: &SimpleBackendConfig,
        quit_notification: ServerQuitWatcher,
        state: T,
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
        let mut sending_logic = FcmSendingLogic::new();
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
                .save_current_non_empty_notification_flags_from_cache_to_database()
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
            .get_push_notification_state_info_and_add_notification_value(
                send_push_notification.account_id,
            )
            .await
            .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)?;

        let (info, flags) = match info {
            PushNotificationStateInfoWithFlags::EmptyFlags => return Ok(()),
            PushNotificationStateInfoWithFlags::WithFlags { info, flags } => (info, flags),
        };

        if info.fcm_notification_sent {
            self.state
                .remove_specific_notification_flags_from_cache(
                    send_push_notification.account_id,
                    flags,
                )
                .await
                .change_context(
                    PushNotificationError::RemoveSpecificNotificationFlagsFromCacheFailed,
                )?;
            return Ok(());
        }

        let token = match info.fcm_device_token {
            Some(token) => token,
            None => {
                self.state
                    .remove_specific_notification_flags_from_cache(
                        send_push_notification.account_id,
                        flags,
                    )
                    .await
                    .change_context(
                        PushNotificationError::RemoveSpecificNotificationFlagsFromCacheFailed,
                    )?;
                return Ok(());
            }
        };

        let messages = self
            .state
            .convert_to_push_notifications(send_push_notification.account_id, token, flags)
            .await
            .change_context(PushNotificationError::CreatingPushNotificationsFailed)?;

        for m in messages {
            if let Some(m) = m.message {
                match sending_logic.send_push_notification(m, fcm).await {
                    Ok(()) => (),
                    Err(action) => match action {
                        UnusualAction::DisablePushNotificationSupport => {
                            self.fcm = None;
                            return Ok(());
                        }
                        UnusualAction::RemoveDeviceToken => {
                            return self
                                .state
                                .remove_device_token(send_push_notification.account_id)
                                .await
                                .change_context(PushNotificationError::RemoveDeviceTokenFailed);
                        }
                    },
                }
            }

            self.state
                .remove_specific_notification_flags_from_cache(
                    send_push_notification.account_id,
                    m.flags,
                )
                .await
                .change_context(
                    PushNotificationError::RemoveSpecificNotificationFlagsFromCacheFailed,
                )?;

            if let Some(action) = m.successful_sending_action {
                self.state
                    .handle_successful_message_sending_action(action)
                    .await?;
            }
        }

        self.state
            .enable_push_notification_sent_flag(send_push_notification.account_id)
            .await
            .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)?;

        Ok(())
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

    async fn retry_sending(&mut self, message: &Message, fcm: &FcmClient) -> NextAction {
        match (self.forced_wait_time.take(), self.exponential_backoff) {
            (None, None) =>
            // First time trying to send this message.
            // Basic rate limiting might be good, so wait some time.
            {
                tokio::time::sleep(Duration::from_millis(self.initial_send_rate_limit_millis)).await
            }
            (Some(forced_wait_time), _) => tokio::time::sleep(forced_wait_time).await,
            (_, Some(exponential_backoff)) => {
                // TODO: Add some jitter time?
                let next_exponential_backoff =
                    exponential_backoff.as_millis() * exponential_backoff.as_millis();
                tokio::time::sleep(exponential_backoff).await;
                self.exponential_backoff =
                    Some(Duration::from_millis(next_exponential_backoff as u64));
            }
        }

        match fcm.send(message).await {
            Ok(response) => {
                let action = response.recommended_error_handling_action();
                if let Some(action) = &action {
                    error!(
                        "FCM error detected, response: {:#?}, action: {:#?}",
                        response, action
                    );
                }
                match action {
                    None => {
                        // TODO(prod): Remove logging
                        info!("FCM send successful");
                        NextAction::NextMessage // No errors
                    }
                    Some(
                        RecomendedAction::CheckIosAndWebCredentials
                        | RecomendedAction::CheckSenderIdEquality,
                    ) => {
                        error!(
                            "Disabling FCM support because of recomended action: {:?}",
                            action
                        );
                        NextAction::UnusualAction(UnusualAction::DisablePushNotificationSupport)
                    }
                    Some(RecomendedAction::FixMessageContent) => {
                        // Handle iOS only APNs BadDeviceToken error.
                        // After the error next FCM message sending will
                        // fail with FcmResponseError::Unregistered.
                        let bad_device_token_error = response
                            .json()
                            .get("error")
                            .and_then(|v| v.as_object())
                            .and_then(|v| v.get("details"))
                            .and_then(|v| v.as_array())
                            .and_then(|v| {
                                v.iter().filter_map(|v| v.as_object()).find(|v| {
                                    v.get("reason")
                                        == Some(&Value::String("BadDeviceToken".to_string()))
                                })
                            });
                        if bad_device_token_error.is_some() {
                            error!("APNs BadDeviceToken error");
                            // Use the current Firebase device token for the
                            // next message because it is not documented that
                            // next FCM message sending will return
                            // FcmResponseError::Unregistered error.
                            NextAction::NextMessage
                        } else {
                            error!(
                                "Disabling FCM support because of recomended action: {:?}",
                                action
                            );
                            NextAction::UnusualAction(UnusualAction::DisablePushNotificationSupport)
                        }
                    }
                    Some(RecomendedAction::RemoveFcmAppToken) => {
                        NextAction::UnusualAction(UnusualAction::RemoveDeviceToken)
                    }
                    Some(RecomendedAction::ReduceMessageRateAndRetry(wait_time)) => {
                        self.initial_send_rate_limit_millis *= 2;
                        self.handle_recommended_wait_time(wait_time);
                        NextAction::Retry
                    }
                    Some(RecomendedAction::Retry(wait_time)) => {
                        self.handle_recommended_wait_time(wait_time);
                        NextAction::Retry
                    }
                    Some(RecomendedAction::HandleUnknownError) => {
                        error!("FCM unknown error");
                        // Just set forced wait time and hope for the best...
                        warn!("Waiting 60 seconds before retrying message sending");
                        self.forced_wait_time = Some(Duration::from_secs(60));
                        NextAction::Retry
                    }
                }
            }
            Err(e) => {
                error!("FCM send failed: {:?}", e);
                if e.is_access_token_missing_even_if_server_requests_completed() {
                    error!("Disabling FCM support because service account key might be invalid");
                    NextAction::UnusualAction(UnusualAction::DisablePushNotificationSupport)
                } else {
                    // Just set forced wait time and hope for the best...
                    warn!("Waiting 60 seconds before retrying message sending");
                    self.forced_wait_time = Some(Duration::from_secs(60));
                    NextAction::Retry
                }
            }
        }
    }

    fn handle_recommended_wait_time(&mut self, recommendation: RecomendedWaitTime) {
        match recommendation {
            RecomendedWaitTime::InitialWaitTime(wait_time) => {
                if self.exponential_backoff.is_none() {
                    // Set initial time for exponential back-off
                    self.exponential_backoff = Some(wait_time);
                }
            }
            RecomendedWaitTime::SpecificWaitTime(retry_after) => {
                self.forced_wait_time = Some(retry_after.wait_time())
            }
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

// TODO(prod): Limit push notification sending rate.
//             Only one push notification should be sent
//             per conversation until user opens the app.
//             Same for other types of notifications.
//             At least likes.
//             Or is limiting pending message count enough
//             for message push notifications?
// TODO(prod): Push notifications for likes and image moderation updates
