use a2::{
    Client, ClientConfig, CollapseId, DefaultNotificationBuilder, Endpoint, Error,
    NotificationBuilder, NotificationOptions, request::payload::Payload,
};
use config::Config;
use error_stack::{Result, ResultExt};
use model::{FcmDeviceToken, PushNotification};
use simple_backend_config::file::ApnsConfig;
use tracing::{error, info};

use crate::push_notifications::{
    PushNotificationError, PushNotificationStateProvider, SendPushNotification,
};

struct InternalState {
    client: Client,
    topic: String,
}

pub struct ApnsManager {
    state: Option<InternalState>,
    sending_logic: ApnsSendingLogic,
}

impl ApnsManager {
    fn new_internal_state(config: &ApnsConfig) -> Result<InternalState, PushNotificationError> {
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

        Ok(InternalState {
            client,
            topic: config.ios_bundle_id.clone(),
        })
    }

    pub async fn new(config: &Config) -> Self {
        let apns = if let Some(config) = config.simple_backend().apns_config() {
            match Self::new_internal_state(config) {
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

        Self {
            state: apns,
            sending_logic,
        }
    }

    pub async fn send_apns_notification(
        &mut self,
        event: SendPushNotification,
        state: &impl PushNotificationStateProvider,
    ) -> Result<(), PushNotificationError> {
        let internal_state = if let Some(internal_state) = &self.state {
            internal_state
        } else {
            return Ok(());
        };

        let info = state
            .get_and_reset_push_notifications(event.account_id)
            .await
            .change_context(PushNotificationError::ReadingNotificationSentStatusFailed)?;

        let Some(token) = info.db_state.fcm_device_token else {
            return Ok(());
        };

        for n in info.notifications {
            let Some(title) = n.title() else {
                // Hiding notifications is not supported
                continue;
            };

            let notification =
                self.create_notification(&token, &n, title, &internal_state.topic)?;

            match self
                .sending_logic
                .send_push_notification(&internal_state.client, notification)
                .await
            {
                Ok(()) => (),
                Err(action) => match action {
                    UnusualAction::DisablePushNotificationSupport => {
                        self.state = None;
                        return Ok(());
                    }
                    UnusualAction::RemoveDeviceToken => {
                        return state
                            .remove_device_token(event.account_id)
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
        token: &'a FcmDeviceToken,
        notification: &'a PushNotification,
        title: &'a str,
        apns_topic: &'a str,
    ) -> Result<Payload<'a>, PushNotificationError> {
        let mut builder = DefaultNotificationBuilder::new()
            .set_title(title)
            .set_sound("default");

        if let Some(body) = notification.body() {
            builder = builder.set_body(body);
        }

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
            .add_custom_data("a", &notification.a())
            .change_context(PushNotificationError::Serialize)?;
        payload
            .add_custom_data("data", &notification.data())
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
        match apns.send(notification).await {
            Ok(_) => {
                if self.debug_logging {
                    info!("APNs send successful");
                }
                Ok(())
            }
            Err(Error::ResponseError(response)) => {
                match response.code {
                    410 => Err(UnusualAction::RemoveDeviceToken),
                    400 | 403 | 405 | 413 => {
                        error!(
                            "APNs send failed: status: {}, disabling APNs notifications",
                            response.code
                        );
                        Err(UnusualAction::DisablePushNotificationSupport)
                    }
                    429 => {
                        // TODO(prod): Retry with delay
                        Ok(())
                    }
                    500 | 503 => {
                        // TODO(prod): Retry after 15 minutes
                        Ok(())
                    }
                    _ => {
                        // Unknown error
                        error!("APNs send failed: status: {}", response.code);
                        Ok(())
                    }
                }
            }
            Err(e) => {
                error!("APNs send failed: {e}");
                Ok(())
            }
        }
    }
}

pub enum UnusualAction {
    DisablePushNotificationSupport,
    RemoveDeviceToken,
}
