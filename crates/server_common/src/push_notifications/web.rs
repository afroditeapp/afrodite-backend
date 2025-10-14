use std::time::Duration;

use config::Config;
use error_stack::{Result, ResultExt};
use model::{PushNotification, PushNotificationDeviceToken};
use simple_backend::ServerQuitWatcher;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{error, info, warn};
use web_push::{
    ContentEncoding, HyperWebPushClient, PartialVapidSignatureBuilder, SubscriptionInfo, Urgency,
    WebPushClient, WebPushError, WebPushMessageBuilder,
};

use crate::push_notifications::{
    PushNotificationError, PushNotificationStateProvider, SendPushNotification,
};

struct WebPushClientState {
    client: HyperWebPushClient,
    vapid_builder: PartialVapidSignatureBuilder,
}

pub struct WebPushManager<T> {
    web: Option<WebPushClientState>,
    sending_logic: WebPushSendingLogic,
    receiver: Receiver<SendPushNotification>,
    state: T,
}

pub struct WebPushManagerQuitHandle {
    task: JoinHandle<()>,
}

impl WebPushManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("WebPushManager quit failed. Error: {e:?}");
            }
        }
    }
}

impl<T: PushNotificationStateProvider + Send + Sync + 'static> WebPushManager<T> {
    pub async fn new_manager(
        config: &Config,
        receiver: Receiver<SendPushNotification>,
        state: T,
        quit_notification: ServerQuitWatcher,
    ) -> WebPushManagerQuitHandle {
        let web = if let Some((_, vapid_builder)) = config.simple_backend().web_push_config() {
            Some(WebPushClientState {
                client: HyperWebPushClient::new(),
                vapid_builder: vapid_builder.clone(),
            })
        } else {
            None
        };

        let debug_logging = config
            .simple_backend()
            .web_push_config()
            .map(|v| v.0.debug_logging)
            .unwrap_or_default();
        let sending_logic = WebPushSendingLogic::new(debug_logging);

        let mut manager = Self {
            web,
            sending_logic,
            receiver,
            state,
        };

        let task = tokio::spawn(async move {
            manager.run(quit_notification).await;
        });

        WebPushManagerQuitHandle { task }
    }

    pub async fn run(&mut self, mut quit_notification: ServerQuitWatcher) {
        loop {
            tokio::select! {
                notification = self.receiver.recv() => {
                    match notification {
                        Some(notification) => match self.handle_notification(notification).await {
                            Ok(()) => (),
                            Err(e) => {
                                error!("Web push notification handling failed: {e:?}");
                            }
                        },
                        None => {
                            warn!("Web push notification channel is broken");
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
        let web = if let Some(web) = &self.web {
            web
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

        for n in info.notifications {
            if n.title().is_none() {
                // Hiding notfications is not supported on web
                continue;
            }

            let notification_data = self.create_notification(&n)?;

            match self
                .sending_logic
                .send_push_notification(
                    &web.client,
                    &web.vapid_builder,
                    &token,
                    &notification_data,
                    n.id(),
                )
                .await
            {
                Ok(()) => (),
                Err(action) => match action {
                    UnusualAction::DisablePushNotificationSupport => {
                        self.web = None;
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

    fn create_notification(
        &self,
        notification: &PushNotification,
    ) -> Result<String, PushNotificationError> {
        let notification_json = serde_json::json!({
            "id": notification.id(),
            "title": notification.title(),
            "body": notification.body(),
        });

        serde_json::to_string(&notification_json).change_context(PushNotificationError::Serialize)
    }
}

struct WebPushSendingLogic {
    debug_logging: bool,
}

impl WebPushSendingLogic {
    pub fn new(debug_logging: bool) -> Self {
        Self { debug_logging }
    }

    pub async fn send_push_notification(
        &mut self,
        client: &HyperWebPushClient,
        vapid_builder: &PartialVapidSignatureBuilder,
        token: &PushNotificationDeviceToken,
        notification_data: &str,
        topic: &str,
    ) -> std::result::Result<(), UnusualAction> {
        let mut retry_once_done = false;
        loop {
            match self
                .send_push_notification_internal(
                    client,
                    vapid_builder,
                    token,
                    notification_data,
                    topic,
                )
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
        client: &HyperWebPushClient,
        vapid_builder: &PartialVapidSignatureBuilder,
        token: &PushNotificationDeviceToken,
        notification_data: &str,
        topic: &str,
    ) -> std::result::Result<(), Action> {
        let subscription_info: SubscriptionInfo =
            serde_json::from_str(token.as_str()).map_err(|e| {
                error!("Failed to parse subscription info: {e}");
                Action::RemoveDeviceToken
            })?;

        let vapid_signature = vapid_builder
            .clone()
            .add_sub_info(&subscription_info)
            .build()
            .map_err(|e| {
                error!("Failed to build VAPID signature: {e}");
                Action::DisablePushNotificationSupport
            })?;

        let mut message_builder = WebPushMessageBuilder::new(&subscription_info);
        message_builder.set_ttl(60 * 60 * 24 * 14);
        message_builder.set_urgency(Urgency::High);
        message_builder.set_topic(topic.to_string());
        message_builder.set_payload(ContentEncoding::Aes128Gcm, notification_data.as_bytes());
        message_builder.set_vapid_signature(vapid_signature);

        let message = message_builder.build().map_err(|e| {
            error!("Message building failed: {e}");
            Action::DisablePushNotificationSupport
        })?;

        match client.send(message).await {
            Ok(()) => {
                if self.debug_logging {
                    info!("Web push notification send successful");
                }
                Ok(())
            }
            Err(e) => {
                match e {
                    WebPushError::EndpointNotValid(_) | WebPushError::EndpointNotFound(_) => {
                        Err(Action::RemoveDeviceToken)
                    }
                    WebPushError::Io(_)
                    | WebPushError::InvalidUri
                    | WebPushError::InvalidTtl
                    | WebPushError::InvalidTopic
                    | WebPushError::InvalidClaims
                    | WebPushError::Unauthorized(_)
                    | WebPushError::BadRequest(_)
                    | WebPushError::PayloadTooLarge
                    | WebPushError::InvalidPackageName
                    | WebPushError::MissingCryptoKeys
                    | WebPushError::InvalidCryptoKeys => {
                        error!("Disabling web push notifications, error: {e}");
                        Err(Action::DisablePushNotificationSupport)
                    }
                    WebPushError::ServerError { retry_after, info } => {
                        if let Some(retry_after) = retry_after {
                            tokio::time::sleep(retry_after).await;
                            Err(Action::Retry)
                        } else {
                            error!("ServerError, info: {info}");
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            Err(Action::RetryOnce)
                        }
                    }
                    WebPushError::Unspecified
                    | WebPushError::InvalidResponse
                    | WebPushError::Other(_) => {
                        error!("Error: {e}");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        Err(Action::RetryOnce)
                    }
                    WebPushError::ResponseTooLarge => {
                        error!("Error: {e}");
                        // Assume that sending is successful
                        Ok(())
                    }
                    WebPushError::NotImplemented(_) => {
                        error!("Error: {e}");
                        // Ignore error
                        Ok(())
                    }
                }
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
