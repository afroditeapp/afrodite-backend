use std::time::Duration;

use chrono::{Datelike, Utc, Weekday};
use config::file::AutomaticProfileSearchConfig;
use model::{NotificationEvent, WeekdayFlags};
use model_media::ProfileIteratorSessionId;
use model_profile::AccountIdInternal;
use server_api::{
    app::{EventManagerProvider, GetConfig, ReadData, WriteData},
    db_write_raw,
};
use server_common::result::{Result, WrappedResultExt};
use server_data::read::GetReadCommandsCommon;
use server_data_profile::{
    read::GetReadProfileCommands,
    write::GetWriteCommandsProfile,
};
use server_state::S;
use simple_backend::ServerQuitWatcher;
use simple_backend_utils::time::{seconds_until_current_time_is_at, sleep_until_current_time_is_at};
use tokio::{task::JoinHandle, time::sleep};
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
enum ProfileSearchError {
    #[error("Sleep until next run of scheduled tasks failed")]
    TimeError,

    #[error("Database error")]
    DatabaseError,

    #[error("Too many accounts")]
    AccountCount,

    #[error("Concurrent write command error")]
    ConcurrentWriteCommand,

    #[error("Event sending failed")]
    EventSending,
}

#[derive(Debug)]
pub struct ProfileSearchManagerQuitHandle {
    task: JoinHandle<()>,
}

impl ProfileSearchManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("ProfileSearchkManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct ProfileSearchManager {
    state: S,
}

impl ProfileSearchManager {
    pub fn new_manager(
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> ProfileSearchManagerQuitHandle {
        let manager = Self { state };

        let task = tokio::spawn(manager.run(quit_notification));

        ProfileSearchManagerQuitHandle { task }
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        let mut check_cooldown = false;
        let config = self.state.config().automatic_profile_search();

        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(120)), if check_cooldown => {
                    check_cooldown = false;
                }
                result = Self::sleep_until(config), if !check_cooldown => {
                    match result {
                        Ok(()) => {
                            self.run_tasks(&mut quit_notification, config).await;
                        },
                        Err(e) => {
                            warn!("Sleep until failed. Error: {:?}", e);
                        }
                    }
                    check_cooldown = true;
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    async fn sleep_until(config: &AutomaticProfileSearchConfig) -> Result<(), ProfileSearchError> {
        sleep_until_current_time_is_at(config.daily_start_time)
            .await
            .change_context(ProfileSearchError::TimeError)?;
        Ok(())
    }

    async fn run_tasks(
        &self,
        quit_notification: &mut ServerQuitWatcher,
        config: &AutomaticProfileSearchConfig,
    ) {
        tokio::select! {
            r = self.run_tasks_and_return_result(config) => {
                match r {
                    Ok(()) => (),
                    Err(e) => {
                        error!("Automatic profile search failed, error: {:?}", e);
                    }
                }
            }
            _ = quit_notification.recv() => (),
        }
    }

    async fn run_tasks_and_return_result(
        &self,
        config: &AutomaticProfileSearchConfig,
    ) -> Result<(), ProfileSearchError> {
        let milliseconds = seconds_until_current_time_is_at(config.daily_end_time)
            .change_context(ProfileSearchError::TimeError)? * 1000;

        let accounts = self
            .state
            .read()
            .common()
            .account_ids_for_logged_in_clients()
            .await;

        let account_count = TryInto::<u64>::try_into(accounts.len())
            .change_context(ProfileSearchError::AccountCount)?;

        let time_for_each_account = milliseconds / account_count;

        for a in accounts {
            self.handle_account(a).await?;
            tokio::time::sleep(Duration::from_millis(time_for_each_account)).await;
        }

        Ok(())
    }

    async fn handle_account(
        &self,
        account: AccountIdInternal,
    ) -> Result<(), ProfileSearchError> {
        let settings = self
            .state
            .read()
            .profile()
            .notification()
            .chat_app_notification_settings(account)
            .await
            .change_context(ProfileSearchError::DatabaseError)?;

        if !settings.automatic_profile_search {
            return Ok(());
        }

        let current_weekday = match Utc::now().weekday() {
            Weekday::Mon => WeekdayFlags::MONDAY,
            Weekday::Tue => WeekdayFlags::TUESDAY,
            Weekday::Wed => WeekdayFlags::WEDNESDAY,
            Weekday::Thu => WeekdayFlags::THURSDAY,
            Weekday::Fri => WeekdayFlags::FRIDAY,
            Weekday::Sat => WeekdayFlags::SATURDAY,
            Weekday::Sun => WeekdayFlags::SUNDAY,
        };
        let selected_weekdays: WeekdayFlags = settings.automatic_profile_search_weekdays.into();
        if !selected_weekdays.contains(current_weekday) {
            return Ok(());
        }

        let Some(last_seen_time) = self
            .state
            .read()
            .profile()
            .profile(account)
            .await
            .change_context(ProfileSearchError::DatabaseError)?
            .last_seen_time
            .and_then(|v| v.last_seen_unix_time()) else {
                return Ok(());
            };

        db_write_raw!(self.state, move |cmds| {
            cmds.profile()
                .set_automatic_profile_search_last_seen_time(account, last_seen_time)
                .await
        })
            .await
            .change_context(ProfileSearchError::DatabaseError)?;

        let Some(data) = self
            .state
            .concurrent_write_profile_blocking(account.as_id(), move |cmds| {
                let iterator_session_id: ProfileIteratorSessionId = cmds.automatic_profile_search_reset_profile_iterator(account)?.into();
                cmds.automatic_profile_search_next_profiles(account, iterator_session_id)
            })
            .await
            .change_context(ProfileSearchError::ConcurrentWriteCommand)?
            .change_context(ProfileSearchError::ConcurrentWriteCommand)? else {
                return Ok(());
            };

        if data.is_empty() {
            return Ok(());
        }

        db_write_raw!(self.state, move |cmds| {
            cmds.profile_admin()
                .notification()
                .show_automatic_profile_search_notification(account)
                .await
        })
            .await
            .change_context(ProfileSearchError::DatabaseError)?;

        self
            .state
            .event_manager()
            .send_notification(account, NotificationEvent::AutomaticProfileSearchCompleted)
            .await
            .change_context(ProfileSearchError::EventSending)?;

        Ok(())
    }
}
