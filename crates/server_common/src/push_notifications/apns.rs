use std::time::Duration;

use a2::{
    Client, ClientConfig, CollapseId, DefaultNotificationBuilder, Endpoint, Error,
    NotificationBuilder, NotificationOptions, request::payload::Payload,
};
use aes_gcm::{
    AeadCore, Aes128Gcm, KeyInit,
    aead::{Aead, OsRng},
};
use base64::Engine;
use config::Config;
use error_stack::{Report, Result, ResultExt};
use model::{PushNotification, PushNotificationDeviceToken};
use simple_backend::ServerQuitWatcher;
use simple_backend_config::file::ApnsConfig;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{error, info, warn};

use crate::push_notifications::{
    PushNotificationError, PushNotificationStateProvider, SendPushNotification,
};

struct ApnsClient {
    client: Client,
    topic: String,
}

pub struct ApnsManager<T> {
    apns: Option<ApnsClient>,
    sending_logic: ApnsSendingLogic,
    receiver: Receiver<SendPushNotification>,
    state: T,
}

pub struct ApnsManagerQuitHandle {
    task: JoinHandle<()>,
}

impl ApnsManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("ApnsManager quit failed. Error: {e:?}");
            }
        }
    }
}

impl<T: PushNotificationStateProvider + Send + Sync + 'static> ApnsManager<T> {
    fn new_client(config: &ApnsConfig) -> Result<ApnsClient, PushNotificationError> {
        let file = std::fs::File::open(&config.key_path)
            .change_context(PushNotificationError::CreateApnsClient)?;

        let endpoint = if config.production_servers {
            Endpoint::Production
        } else {
            Endpoint::Sandbox
        };

        let client = Client::token(
            file,
            &config.key_id,
            &config.team_id,
            ClientConfig::new(endpoint),
        )
        .change_context(PushNotificationError::CreateApnsClient)?;

        Ok(ApnsClient {
            client,
            topic: config.ios_bundle_id.clone(),
        })
    }

    pub async fn new_manager(
        config: &Config,
        receiver: Receiver<SendPushNotification>,
        state: T,
        quit_notification: ServerQuitWatcher,
    ) -> ApnsManagerQuitHandle {
        let apns = if let Some(config) = config.simple_backend().apns_config() {
            match Self::new_client(config) {
                Ok(v) => Some(v),
                Err(e) => {
                    error!("Internal state creation failed: {e:?}");
                    None
                }
            }
        } else {
            None
        };

        let debug_logging = config
            .simple_backend()
            .apns_config()
            .map(|v| v.debug_logging)
            .unwrap_or_default();
        let sending_logic = ApnsSendingLogic::new(debug_logging);

        let mut manager = Self {
            apns,
            sending_logic,
            receiver,
            state,
        };

        let task = tokio::spawn(async move {
            manager.run(quit_notification).await;
        });

        ApnsManagerQuitHandle { task }
    }

    pub async fn run(&mut self, mut quit_notification: ServerQuitWatcher) {
        loop {
            tokio::select! {
                notification = self.receiver.recv() => {
                    match notification {
                        Some(notification) => match self.handle_notification(notification).await {
                            Ok(()) => (),
                            Err(e) => {
                                error!("APNs notification handling failed: {e:?}");
                            }
                        },
                        None => {
                            warn!("APNs notification channel is broken");
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
        let apns = if let Some(apns) = &self.apns {
            apns
        } else {
            return Ok(());
        };

        let info = self
            .state
            .get_and_reset_push_notifications(send_push_notification.account_id)
            .await
            .change_context(PushNotificationError::ReadingNotificationSentStatusFailed)?;

        let Some(token) = info.db_state.push_notification_device_token else {
            return Ok(());
        };

        let Some(encryption_key) = info.db_state.push_notification_encryption_key else {
            return Ok(());
        };

        let encryption_key_bytes = base64::engine::general_purpose::STANDARD
            .decode(encryption_key.as_str())
            .change_context(PushNotificationError::EncryptionFailed)?;

        for n in info.notifications {
            let Some(title) = n.title() else {
                // Hiding notifications is not supported
                continue;
            };

            let notification =
                self.create_notification(&token, &n, title, &apns.topic, &encryption_key_bytes)?;

            match self
                .sending_logic
                .send_push_notification(&apns.client, notification)
                .await
            {
                Ok(()) => (),
                Err(action) => match action {
                    UnusualAction::DisablePushNotificationSupport => {
                        self.apns = None;
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

    fn create_notification<'a>(
        &self,
        token: &'a PushNotificationDeviceToken,
        notification: &'a PushNotification,
        title: &'a str,
        apns_topic: &'a str,
        encryption_key_bytes: &[u8],
    ) -> Result<Payload<'a>, PushNotificationError> {
        let notification_content = if let Some(body) = notification.body() {
            serde_json::json!({
                "title": title,
                "body": body,
            })
        } else {
            serde_json::json!({
                "title": title,
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

        let builder = DefaultNotificationBuilder::new()
            .set_title("Notification decrypting failed")
            .set_sound("default")
            .set_mutable_content();

        let collapse_id = CollapseId::new(notification.id())
            .change_context(PushNotificationError::NotificationBuildingFailed)?;

        let options = NotificationOptions {
            apns_collapse_id: Some(collapse_id),
            apns_topic: Some(apns_topic),
            ..Default::default()
        };

        let mut payload = builder.build(token.as_str(), options);
        payload
            .add_custom_data("id", &notification.id())
            .change_context(PushNotificationError::Serialize)?;
        payload
            .add_custom_data("encrypted", &encrypted_base64)
            .change_context(PushNotificationError::Serialize)?;
        payload
            .add_custom_data("nonce", &nonce_base64)
            .change_context(PushNotificationError::Serialize)?;
        Ok(payload)
    }
}

struct ApnsSendingLogic {
    debug_logging: bool,
}

impl ApnsSendingLogic {
    pub fn new(debug_logging: bool) -> Self {
        Self { debug_logging }
    }

    pub async fn send_push_notification(
        &mut self,
        apns: &Client,
        notification: Payload<'_>,
    ) -> std::result::Result<(), UnusualAction> {
        let mut retry_once_done = false;
        loop {
            match self
                .send_push_notification_internal(apns, notification.clone())
                .await
            {
                Ok(()) => return Ok(()),
                Err(Action::DisablePushNotificationSupport) => {
                    return Err(UnusualAction::DisablePushNotificationSupport);
                }
                Err(Action::RemoveDeviceToken) => return Err(UnusualAction::RemoveDeviceToken),
                Err(Action::Retry) => (),
                Err(Action::RetryOnce) => {
                    if retry_once_done {
                        return Ok(());
                    }
                    retry_once_done = true;
                }
            }
        }
    }

    async fn send_push_notification_internal(
        &mut self,
        apns: &Client,
        notification: Payload<'_>,
    ) -> std::result::Result<(), Action> {
        match apns.send(notification).await {
            Ok(_) => {
                if self.debug_logging {
                    info!("APNs send successful");
                }
                Ok(())
            }
            Err(Error::ResponseError(response)) => {
                match response.code {
                    410 => Err(Action::RemoveDeviceToken),
                    400 | 403 | 405 | 413 => {
                        error!(
                            "APNs send failed: status: {}, disabling APNs notifications",
                            response.code
                        );
                        Err(Action::DisablePushNotificationSupport)
                    }
                    429 => {
                        // Too many messages sent to a single device.
                        warn!(
                            "APNs send failed: status: {}, retrying sending",
                            response.code
                        );
                        // APNs docs don't have specific wait time for this case.
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        Err(Action::Retry)
                    }
                    500 | 503 => {
                        warn!(
                            "APNs send failed: status: {}, retrying sending",
                            response.code
                        );
                        // Wait time is from APNs docs
                        tokio::time::sleep(Duration::from_secs(60 * 15)).await;
                        Err(Action::Retry)
                    }
                    _ => {
                        // Unknown error
                        error!("APNs send failed: status: {}", response.code);
                        Ok(())
                    }
                }
            }
            Err(e @ Error::ClientError(_))
            | Err(e @ Error::ConnectionError(_))
            | Err(e @ Error::RequestTimeout(_)) => {
                warn!("APNs send failed: {e}, retrying sending");
                tokio::time::sleep(Duration::from_secs(1)).await;
                Err(Action::RetryOnce)
            }
            Err(e) => {
                error!("APNs send failed: {e}, disabling APNs notifications");
                Err(Action::DisablePushNotificationSupport)
            }
        }
    }
}

pub enum UnusualAction {
    DisablePushNotificationSupport,
    RemoveDeviceToken,
}

pub enum Action {
    DisablePushNotificationSupport,
    RemoveDeviceToken,
    Retry,
    RetryOnce,
}
