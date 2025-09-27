use std::time::Duration;

use config::Config;
use error_stack::{Result, ResultExt};
use fcm::{
    FcmClient,
    message::{AndroidConfig, AndroidMessagePriority, ApnsConfig, Message, Target},
    response::{RecomendedAction, RecomendedWaitTime},
};
use model::PushNotificationStateInfoWithFlags;
use rand::{Rng, rngs::OsRng};
use serde_json::{Value, json};
use tracing::{error, info, warn};

use crate::push_notifications::{
    PushNotificationError, PushNotificationStateProvider, SendPushNotification,
};

pub struct FcmManager {
    fcm: Option<FcmClient>,
    sending_logic: FcmSendingLogic,
}

impl FcmManager {
    pub async fn new(config: &Config) -> Self {
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

        let debug_logging = config
            .simple_backend()
            .firebase_cloud_messaging_config()
            .map(|v| v.debug_logging)
            .unwrap_or_default();
        let sending_logic = FcmSendingLogic::new(debug_logging);

        Self { fcm, sending_logic }
    }

    pub async fn send_fcm_notification(
        &mut self,
        send_push_notification: SendPushNotification,
        state: &impl PushNotificationStateProvider,
    ) -> Result<(), PushNotificationError> {
        let fcm = if let Some(fcm) = &self.fcm {
            fcm
        } else {
            return Ok(());
        };

        let info = state
            .get_and_reset_push_notifications(send_push_notification.account_id)
            .await
            .change_context(PushNotificationError::ReadingNotificationSentStatusFailed)?;

        let Some(token) = info.db_state.fcm_device_token else {
            return Ok(());
        };

        let is_visible = true;

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

        match self.sending_logic.send_push_notification(m, fcm).await {
            Ok(()) => Ok(()),
            Err(action) => match action {
                UnusualAction::DisablePushNotificationSupport => {
                    self.fcm = None;
                    Ok(())
                }
                UnusualAction::RemoveDeviceToken => state
                    .remove_device_token(send_push_notification.account_id)
                    .await
                    .change_context(PushNotificationError::RemoveDeviceTokenFailed),
            },
        }
    }
}

pub struct FcmSendingLogic {
    initial_send_rate_limit_millis: u64,
    exponential_backoff: Option<Duration>,
    forced_wait_time: Option<Duration>,
    debug_logging: bool,
}

impl FcmSendingLogic {
    pub fn new(debug_logging: bool) -> Self {
        Self {
            initial_send_rate_limit_millis: 1,
            exponential_backoff: None,
            forced_wait_time: None,
            debug_logging,
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
                let next_exponential_backoff =
                    exponential_backoff.as_millis() * exponential_backoff.as_millis();
                let jitter = Duration::from_millis(OsRng.gen_range(0..=1000));
                tokio::time::sleep(exponential_backoff + jitter).await;
                self.exponential_backoff =
                    Some(Duration::from_millis(next_exponential_backoff as u64));
            }
        }

        match fcm.send(message).await {
            Ok(response) => {
                let action = response.recommended_error_handling_action();
                if self.debug_logging
                    && let Some(action) = &action
                {
                    error!("FCM error detected, response: {response:#?}, action: {action:#?}");
                }
                match action {
                    None => {
                        if self.debug_logging {
                            info!("FCM send successful");
                        }
                        NextAction::NextMessage // No errors
                    }
                    Some(
                        RecomendedAction::CheckIosAndWebCredentials
                        | RecomendedAction::CheckSenderIdEquality,
                    ) => {
                        error!("Disabling FCM support because of recomended action: {action:?}");
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

enum NextAction {
    UnusualAction(UnusualAction),
    NextMessage,
    Retry,
}

pub enum UnusualAction {
    DisablePushNotificationSupport,
    RemoveDeviceToken,
}
