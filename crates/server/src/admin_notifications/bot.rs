use std::time::Duration;

use model::{AdminBotNotificationTypes, AdminNotificationTypes, EventToClientInternal};
use server_api::{DataError, app::EventManagerProvider};
use server_common::result::Result;
use server_data::{app::ReadData, read::GetReadCommandsCommon};
use server_state::S;
use tracing::error;

use super::WaitSendTimer;

pub struct BotNotificationManager {
    timer: WaitSendTimer,
    pending_events: AdminBotNotificationTypes,
    state: S,
}

impl BotNotificationManager {
    pub fn new(state: S) -> Self {
        Self {
            timer: WaitSendTimer::new(Duration::from_secs(1)),
            pending_events: AdminBotNotificationTypes::default(),
            state,
        }
    }

    pub async fn handle_notification(&mut self, notification_type: AdminNotificationTypes) {
        let bot_notification = match AdminBotNotificationTypes::try_from(notification_type) {
            Ok(n) => n,
            Err(_) => return, // Not a bot notification type
        };

        if self.timer.timer.is_none() {
            // Timer not running, send immediately
            if let Err(e) = self.send_bot_notifications(&bot_notification).await {
                error!("Failed to send bot notifications: {:?}", e);
            }
            self.timer.start_if_not_running();
        } else {
            // Timer running, add to pending
            self.pending_events = self.pending_events.merge(&bot_notification);
        }
    }

    pub async fn handle_timer_completion(&mut self) {
        if !self.pending_events.is_empty() {
            if let Err(e) = self.send_bot_notifications(&self.pending_events).await {
                error!("Failed to send bot notifications: {:?}", e);
            }
            self.pending_events = AdminBotNotificationTypes::default();
        }
    }

    pub async fn wait_timer_completion(&mut self) {
        self.timer.wait_completion().await;
    }

    async fn send_bot_notifications(
        &self,
        notification: &AdminBotNotificationTypes,
    ) -> Result<(), DataError> {
        let admin_bot_accounts = self
            .state
            .read()
            .common_admin()
            .admin_bot_account_ids()
            .await?;

        for account_id in admin_bot_accounts {
            let event = EventToClientInternal::AdminBotNotification(notification.clone());
            self.state
                .event_manager()
                .send_connected_event(account_id, event)
                .await?;
        }

        Ok(())
    }
}
