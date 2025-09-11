use std::time::Duration;

use model::{AdminNotification, NotificationEvent};
use model_media::{
    GetMediaContentPendingModerationParams, MediaContentType, ModerationQueueType,
    ProfileStringModerationContentType,
};
use model_profile::GetProfileStringPendingModerationParams;
use server_api::app::EventManagerProvider;
use server_common::result::{Result, WrappedResultExt};
use server_data::{app::ReadData, read::GetReadCommandsCommon};
use server_data_media::read::GetReadMediaCommands;
use server_data_profile::read::GetReadProfileCommands;
use server_state::{
    S,
    admin_notifications::{AdminNotificationEvent, AdminNotificationEventReceiver},
    app::AdminNotificationProvider,
};
use simple_backend::ServerQuitWatcher;
use simple_backend_utils::time::seconds_until_current_time_is_at;
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
}

impl AdminNotificationManager {
    pub fn new_manager(
        receiver: AdminNotificationEventReceiver,
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> AdminNotificationManagerQuitHandle {
        let manager = Self { state };

        let task = tokio::spawn(manager.run(receiver, quit_notification));

        AdminNotificationManagerQuitHandle { task }
    }

    async fn run(
        mut self,
        mut receiver: AdminNotificationEventReceiver,
        mut quit_notification: ServerQuitWatcher,
    ) {
        let mut timer = WaitSendTimer::new();
        let mut waiter = StartTimeWaiter::new(self.state.clone());
        waiter.refresh_state().await;

        loop {
            tokio::select! {
                _ = timer.wait_completion() => {
                    if let Err(e) = self.handle_pending_events().await {
                        error!("Error: {:?}", e);
                    }
                }
                _ = waiter.wait_completion() => {
                    waiter.refresh_state().await;
                    timer.start_if_not_running();
                }
                item = receiver.0.recv() => {
                    match item {
                        Some(AdminNotificationEvent::ResetState(id)) => {
                            self.state.admin_notification().write().reset_notification_state(id).await;
                        }
                        Some(AdminNotificationEvent::SendNotificationIfNeeded(_)) => {
                            timer.start_if_not_running();
                        },
                        Some(AdminNotificationEvent::RefreshStartTimeWaiter) => {
                            waiter.refresh_state().await;
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
        // Check all categories, so that every notification contains full info
        let notification = AdminNotification {
            moderate_initial_media_content_bot: self
                .is_initial_content_moderation_needed(true)
                .await?,
            moderate_initial_media_content_human: self
                .is_initial_content_moderation_needed(false)
                .await?,
            moderate_media_content_bot: self.is_content_moderation_needed(true).await?,
            moderate_media_content_human: self.is_content_moderation_needed(false).await?,
            moderate_profile_texts_bot: self
                .is_profile_string_moderation_needed(
                    ProfileStringModerationContentType::ProfileText,
                    true,
                )
                .await?,
            moderate_profile_texts_human: self
                .is_profile_string_moderation_needed(
                    ProfileStringModerationContentType::ProfileText,
                    false,
                )
                .await?,
            moderate_profile_names_bot: self
                .is_profile_string_moderation_needed(
                    ProfileStringModerationContentType::ProfileName,
                    true,
                )
                .await?,
            moderate_profile_names_human: self
                .is_profile_string_moderation_needed(
                    ProfileStringModerationContentType::ProfileName,
                    false,
                )
                .await?,
            process_reports: self.is_report_processing_needed().await?,
        };

        let accounts = self
            .state
            .read()
            .common_admin()
            .notification()
            .get_accounts_which_should_receive_notification(notification.clone())
            .await
            .change_context(AdminNotificationError::DatabaseError)?;

        for (a, settings) in accounts {
            let current_notification_state = self
                .state
                .admin_notification()
                .get_notification_state(a)
                .await
                .unwrap_or_default();

            let new_notification_state =
                current_notification_state.merge(&settings.union(&notification));

            if current_notification_state != new_notification_state {
                self.state
                    .admin_notification()
                    .write()
                    .set_notification_state(a, new_notification_state)
                    .await;
                let r = self
                    .state
                    .event_manager()
                    .send_notification(a, NotificationEvent::AdminNotification)
                    .await;
                if let Err(e) = r {
                    error!("Event sending failed: {:?}", e);
                }
            }
        }

        Ok(())
    }

    async fn is_initial_content_moderation_needed(
        &self,
        is_bot: bool,
    ) -> Result<bool, AdminNotificationError> {
        let values = self
            .state
            .read()
            .media_admin()
            .profile_content_pending_moderation_list(
                is_bot,
                GetMediaContentPendingModerationParams {
                    content_type: MediaContentType::JpegImage,
                    queue: ModerationQueueType::InitialMediaModeration,
                    show_content_which_bots_can_moderate: is_bot,
                },
            )
            .await
            .change_context(AdminNotificationError::DatabaseError)?
            .values;
        Ok(!values.is_empty())
    }

    async fn is_content_moderation_needed(
        &self,
        is_bot: bool,
    ) -> Result<bool, AdminNotificationError> {
        let values = self
            .state
            .read()
            .media_admin()
            .profile_content_pending_moderation_list(
                is_bot,
                GetMediaContentPendingModerationParams {
                    content_type: MediaContentType::JpegImage,
                    queue: ModerationQueueType::MediaModeration,
                    show_content_which_bots_can_moderate: is_bot,
                },
            )
            .await
            .change_context(AdminNotificationError::DatabaseError)?
            .values;
        Ok(!values.is_empty())
    }

    async fn is_profile_string_moderation_needed(
        &self,
        content_type: ProfileStringModerationContentType,
        is_bot: bool,
    ) -> Result<bool, AdminNotificationError> {
        let values = self
            .state
            .read()
            .profile_admin()
            .moderation()
            .profile_string_pending_moderation_list(
                is_bot,
                GetProfileStringPendingModerationParams {
                    content_type,
                    show_values_which_bots_can_moderate: is_bot,
                },
            )
            .await
            .change_context(AdminNotificationError::DatabaseError)?
            .values;
        Ok(!values.is_empty())
    }

    async fn is_report_processing_needed(&self) -> Result<bool, AdminNotificationError> {
        let values = self
            .state
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
        Self { timer: None }
    }

    fn start_if_not_running(&mut self) {
        if self.timer.is_none() {
            const WAIT_TIME: Duration = Duration::from_secs(60);
            let timer =
                tokio::time::interval_at(tokio::time::Instant::now() + WAIT_TIME, WAIT_TIME);
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

struct StartTimeWaiter {
    timer: Option<tokio::time::Interval>,
    state: S,
}

impl StartTimeWaiter {
    fn new(state: S) -> Self {
        Self { timer: None, state }
    }

    async fn refresh_state(&mut self) {
        let start_time = match self
            .state
            .read()
            .common_admin()
            .notification()
            .nearest_start_time()
            .await
        {
            Ok(v) => v,
            Err(e) => {
                error!("Error: {:?}", e);
                return;
            }
        };

        let time_value = start_time.to_utc_time_value();

        let wait_time = match seconds_until_current_time_is_at(time_value) {
            Ok(v) => v,
            Err(e) => {
                error!("Error: {:?}", e);
                return;
            }
        };

        let wait_time = Duration::from_secs(wait_time.max(1));
        let timer = tokio::time::interval_at(tokio::time::Instant::now() + wait_time, wait_time);
        self.timer = Some(timer);
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
