use std::time::Duration;

use aes_gcm::{AeadCore, Aes128Gcm, KeyInit, aead::Aead};
use base64::Engine;
use config::Config;
use error_stack::{Report, Result, ResultExt};
use fcm::{
    FcmClient,
    message::{AndroidConfig, AndroidMessagePriority, Message, Target},
    response::{RecomendedAction, RecomendedWaitTime},
};
use model::{PushNotification, PushNotificationDeviceToken};
use rand::{Rng, rngs::OsRng};
use serde_json::Value;
use simple_backend::ServerQuitWatcher;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{error, info, warn};

use crate::push_notifications::{
    PushNotificationError, PushNotificationStateProvider, SendPushNotification,
};

pub struct FcmManager<T> {
    fcm: Option<FcmClient>,
    sending_logic: FcmSendingLogic,
    receiver: Receiver<SendPushNotification>,
    state: T,
}

pub struct FcmManagerQuitHandle {
    task: JoinHandle<()>,
}

impl FcmManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("FcmManagerQuitHandle quit failed. Error: {e:?}");
            }
        }
    }
}

impl<T: PushNotificationStateProvider + Send + Sync + 'static> FcmManager<T> {
    pub async fn new_manager(
        config: &Config,
        receiver: Receiver<SendPushNotification>,
        state: T,
        quit_notification: ServerQuitWatcher,
    ) -> FcmManagerQuitHandle {
        let fcm = if let Some(fcm_config) = config.simple_backend().fcm_config() {
            // TODO(future): Make possible to use existing reqwest::Client
            //               with FcmClient.
            let fcm_result = FcmClient::builder()
                .service_account_key_json_path(&fcm_config.service_account_key_path)
                .token_cache_json_path(config.simple_backend().fcm_token_cache_path())
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
            .fcm_config()
            .map(|v| v.debug_logging)
            .unwrap_or_default();
        let sending_logic = FcmSendingLogic::new(debug_logging);

        let mut manager = Self {
            fcm,
            sending_logic,
            receiver,
            state,
        };

        let task = tokio::spawn(async move {
            manager.run(quit_notification).await;
        });

        FcmManagerQuitHandle { task }
    }

    pub async fn run(&mut self, mut quit_notification: ServerQuitWatcher) {
        loop {
            tokio::select! {
                notification = self.receiver.recv() => {
                    match notification {
                        Some(notification) => match self.handle_notification(notification).await {
                            Ok(()) => (),
                            Err(e) => {
                                error!("FCM notification handling failed: {e:?}");
                            }
                        },
                        None => {
                            warn!("FCM notification channel is broken");
                            return;
                        }
                    }
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    pub async fn handle_notification(
        &mut self,
        send_push_notification: SendPushNotification,
    ) -> Result<(), PushNotificationError> {
        let fcm = if let Some(fcm) = &self.fcm {
            fcm
        } else {
            return Ok(());
        };

        let info = self
            .state
            .get_and_reset_push_notifications(send_push_notification.account_id)
            .await
            .change_context(PushNotificationError::ReadingNotificationSentStatusFailed)?;

        let Some(token) = info.db_state.device_token else {
            return Ok(());
        };

        let Some(encryption_key) = info.db_state.encryption_key else {
            return Ok(());
        };

        let encryption_key_bytes = base64::engine::general_purpose::STANDARD
            .decode(encryption_key.as_str())
            .change_context(PushNotificationError::EncryptionFailed)?;

        for n in info.notifications {
            let message = self.create_message(&token, &n, &encryption_key_bytes)?;

            match self
                .sending_logic
                .send_push_notification(message, fcm)
                .await
            {
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

        Ok(())
    }

    fn create_message(
        &self,
        token: &PushNotificationDeviceToken,
        notification: &PushNotification,
        encryption_key_bytes: &[u8],
    ) -> Result<Message, PushNotificationError> {
        let notification_content = if let Some(body) = notification.body() {
            serde_json::json!({
                "title": notification.title(),
                "body": body,
            })
        } else {
            serde_json::json!({
                "title": notification.title(),
            })
        };

        let content_json = serde_json::to_string(&notification_content)
            .change_context(PushNotificationError::Serialize)?;

        let cipher = Aes128Gcm::new_from_slice(encryption_key_bytes)
            .change_context(PushNotificationError::EncryptionFailed)?;

        let nonce = Aes128Gcm::generate_nonce(OsRng);

        let encrypted = cipher
            .encrypt(&nonce, content_json.as_bytes())
            .map_err(|_| {
                Report::new(PushNotificationError::EncryptionFailed)
                    .attach_printable("Failed to encrypt notification content")
            })?;

        let encrypted_base64 = base64::engine::general_purpose::STANDARD.encode(&encrypted);
        let nonce_base64 = base64::engine::general_purpose::STANDARD.encode(nonce.as_slice());

        let mut data = serde_json::Map::new();
        data.insert("id".to_string(), serde_json::json!(notification.id()));
        if let Some(channel) = notification.channel() {
            data.insert("channel".to_string(), serde_json::json!(channel));
        }
        data.insert("encrypted".to_string(), serde_json::json!(encrypted_base64));
        data.insert("nonce".to_string(), serde_json::json!(nonce_base64));

        let m = Message {
            data: Some(serde_json::Value::Object(data)),
            target: Target::Token(token.clone().into_string()),
            android: Some(AndroidConfig {
                priority: Some(if notification.is_visible() {
                    AndroidMessagePriority::High
                } else {
                    AndroidMessagePriority::Normal
                }),
                collapse_key: Some(notification.id().to_string()),
                ..Default::default()
            }),
            apns: None,
            webpush: None,
            fcm_options: None,
            notification: None,
        };
        Ok(m)
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
