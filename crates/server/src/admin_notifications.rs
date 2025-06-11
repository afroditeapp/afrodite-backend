use std::time::Duration;

use model::{AdminNotificationSubscriptions, NotificationEvent};
use model_media::{GetProfileContentPendingModerationParams, MediaContentType, ModerationQueueType};
use model_profile::GetProfileTextPendingModerationParams;
use server_api::app::EventManagerProvider;
use server_common::result::{Result, WrappedResultExt};
use server_data::{app::ReadData, read::GetReadCommandsCommon};
use server_data_media::read::GetReadMediaCommands;
use server_data_profile::read::GetReadProfileCommands;
use server_state::{admin_notifications::{AdminNotificationEvent, AdminNotificationEventReceiver}, S, app::AdminNotificationProvider};
use simple_backend::ServerQuitWatcher;
use tokio::task::JoinHandle;
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
enum AdminNotificationError {
    #[error("Database update error")]
    DatabaseError,
}

#[derive(Debug)]
pub struct AdminNotificationManagerQuitHandle {
    task: JoinHandle<()>,
}

impl AdminNotificationManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("AdminNotificationManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct AdminNotificationManager {
    state: S,
    pending_notifications: AdminNotificationSubscriptions,
}

impl AdminNotificationManager {
    pub fn new_manager(
        receiver: AdminNotificationEventReceiver,
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> AdminNotificationManagerQuitHandle {
        let manager = Self {
            state,
            pending_notifications: AdminNotificationSubscriptions::default(),
        };

        let task = tokio::spawn(manager.run(receiver, quit_notification));

        AdminNotificationManagerQuitHandle { task }
    }

    async fn run(
        mut self,
        mut receiver: AdminNotificationEventReceiver,
        mut quit_notification: ServerQuitWatcher,
    ) {
        let mut timer = WaitSendTimer::new();

        loop {
            tokio::select! {
                _ = timer.wait_completion() => {
                    if let Err(e) = self.handle_pending_events().await {
                        error!("Error: {:?}", e);
                    }
                }
                item = receiver.0.recv() => {
                    match item {
                        Some(AdminNotificationEvent::ResetState(id)) => {
                            self.state.admin_notification().reset_notification_state(id).await;
                        }
                        Some(AdminNotificationEvent::SendNotificationIfNeeded(notification)) => {
                            self.pending_notifications.enable(notification);
                            timer.start_if_not_running();
                        },
                        None => {
                            error!("Admin notification manager event channel is broken");
                            return;
                        },
                    }
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    async fn handle_pending_events(&mut self) -> Result<(), AdminNotificationError> {
        if self.pending_notifications.moderate_media_content_bot {
            self.pending_notifications.moderate_media_content_bot =
                self.initial_content_moderation_is_needed(true).await? ||
                self.content_moderation_is_needed(true).await?;
        }

        if self.pending_notifications.moderate_media_content_human {
            self.pending_notifications.moderate_media_content_human =
                self.initial_content_moderation_is_needed(false).await? ||
                self.content_moderation_is_needed(false).await?;
        }

        if self.pending_notifications.moderate_profile_texts_bot {
            self.pending_notifications.moderate_profile_texts_bot =
                self.profile_text_moderation_is_needed(true).await?
        }

        if self.pending_notifications.moderate_profile_texts_human {
            self.pending_notifications.moderate_profile_texts_human =
                self.profile_text_moderation_is_needed(false).await?
        }

        if self.pending_notifications.moderate_profile_names_bot {
            self.pending_notifications.moderate_profile_names_bot =
                self.profile_name_moderation_is_needed().await?
        }

        if self.pending_notifications.moderate_profile_names_human {
            self.pending_notifications.moderate_profile_names_human =
                self.profile_name_moderation_is_needed().await?
        }

        if self.pending_notifications.process_reports {
            self.pending_notifications.process_reports =
                self.report_processing_is_needed().await?
        }

        let accounts = self.state
            .read()
            .common_admin()
            .notification()
            .get_accounts_with_some_wanted_subscriptions(self.pending_notifications.clone())
            .await
            .change_context(AdminNotificationError::DatabaseError)?;

        for (a, _) in accounts {
            if self.state.admin_notification().get_notification_state(a).await != Some(self.pending_notifications.clone()) {
                self.state.admin_notification().set_notification_state(a, self.pending_notifications.clone()).await;
                let r = self.state.event_manager().send_notification(a, NotificationEvent::AdminNotification)
                    .await;
                if let Err(e) = r {
                    error!("Event sending failed: {:?}", e);
                }
            }
        }

        self.pending_notifications = AdminNotificationSubscriptions::default();

        Ok(())
    }

    async fn initial_content_moderation_is_needed(&self, is_bot: bool) -> Result<bool, AdminNotificationError> {
        let values = self.state
            .read()
            .media_admin()
            .profile_content_pending_moderation_list(
                is_bot,
                GetProfileContentPendingModerationParams {
                    content_type: MediaContentType::JpegImage,
                    queue: ModerationQueueType::InitialMediaModeration,
                    show_content_which_bots_can_moderate: is_bot,
                }
            )
            .await
            .change_context(AdminNotificationError::DatabaseError)?
            .values;
        Ok(!values.is_empty())
    }

    async fn content_moderation_is_needed(&self, is_bot: bool) -> Result<bool, AdminNotificationError> {
        let values = self.state
            .read()
            .media_admin()
            .profile_content_pending_moderation_list(
                is_bot,
                GetProfileContentPendingModerationParams {
                    content_type: MediaContentType::JpegImage,
                    queue: ModerationQueueType::MediaModeration,
                    show_content_which_bots_can_moderate: is_bot,
                }
            )
            .await
            .change_context(AdminNotificationError::DatabaseError)?
            .values;
        Ok(!values.is_empty())
    }

    async fn profile_text_moderation_is_needed(&self, is_bot: bool) -> Result<bool, AdminNotificationError> {
        let values = self.state
            .read()
            .profile_admin()
            .profile_text()
            .profile_text_pending_moderation_list(
                is_bot,
                GetProfileTextPendingModerationParams {
                    show_texts_which_bots_can_moderate: is_bot,
                }
            )
            .await
            .change_context(AdminNotificationError::DatabaseError)?
            .values;
        Ok(!values.is_empty())
    }

    // TODO(prod): Add is_bot parameter
    async fn profile_name_moderation_is_needed(&self) -> Result<bool, AdminNotificationError> {
        let values = self.state
            .read()
            .profile_admin()
            .profile_name_allowlist()
            .profile_name_pending_moderation_list()
            .await
            .change_context(AdminNotificationError::DatabaseError)?
            .values;
        Ok(!values.is_empty())
    }

    async fn report_processing_is_needed(&self) -> Result<bool, AdminNotificationError> {
        let values = self.state
            .read()
            .common_admin()
            .report()
            .get_waiting_report_list()
            .await
            .change_context(AdminNotificationError::DatabaseError)?
            .values;
        Ok(!values.is_empty())
    }
}

struct WaitSendTimer {
    timer: Option<tokio::time::Interval>,
}

impl WaitSendTimer {
    fn new() -> Self {
        Self {
            timer: None,
        }
    }

    fn start_if_not_running(&mut self) {
        if self.timer.is_none() {
            const WAIT_TIME: Duration = Duration::from_secs(60);
            let timer = tokio::time::interval_at(tokio::time::Instant::now() + WAIT_TIME, WAIT_TIME);
            self.timer = Some(timer);
        }
    }

    /// Does not return if timer is not running
    async fn wait_completion(&mut self) {
        if let Some(timer) = &mut self.timer {
            timer.tick().await;
            self.timer = None;
        } else {
            std::future::pending().await
        }
    }

}
