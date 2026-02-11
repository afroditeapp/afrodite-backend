use std::time::Duration;

use backup::backup_data;
use model::{ReportTypeNumberInternal, ScheduledTasksConfig, UnixTime};
use model_profile::{
    AccountIdInternal, AccountState, EventToClientInternal, ProfileAge, ProfileUpdateInternal,
};
use server_api::{
    DataError,
    app::{GetConfig, ProfileStatisticsCacheProvider, ReadData, WriteData},
    db_write_raw,
    result::WrappedContextExt,
};
use server_common::result::{Result, WrappedResultExt};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use server_data_profile::{
    read::GetReadProfileCommands, statistics::ProfileStatisticsCacheUtils,
    write::GetWriteCommandsProfile,
};
use server_state::{S, app::ApiLimitsProvider};
use simple_backend::{ServerQuitWatcher, app::PerfCounterDataProvider};
use simple_backend_utils::{IntoReportFromString, time::sleep_until_current_time_is_at};
use tokio::{sync::broadcast::error::TryRecvError, task::JoinHandle, time::sleep};
use tracing::{error, info, warn};

mod backup;
mod email;

#[derive(thiserror::Error, Debug)]
pub enum ScheduledTaskError {
    #[error("Sleep until next run of scheduled tasks failed")]
    TimeError,

    #[error("Database error")]
    DatabaseError,

    #[error("File reading error")]
    FileReadingError,

    #[error("Profile statistics error")]
    ProfileStatisticsError,

    #[error("Unexpected server quit request detected while scheduled tasks were running")]
    QuitRequested,

    #[error("Backup related error")]
    Backup,
}

#[derive(Debug)]
pub struct ScheduledTaskManagerQuitHandle {
    task: JoinHandle<()>,
}

impl ScheduledTaskManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("ScheduledTaskManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct ScheduledTaskManager {
    state: S,
}

impl ScheduledTaskManager {
    pub fn new_manager(
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> ScheduledTaskManagerQuitHandle {
        let manager = Self { state };

        let task = tokio::spawn(manager.run(quit_notification));

        ScheduledTaskManagerQuitHandle { task }
    }

    pub async fn run(self, mut quit_notification: ServerQuitWatcher) {
        let mut check_cooldown = false;
        let config = self.state.config().scheduled_tasks();

        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(120)), if check_cooldown => {
                    check_cooldown = false;
                }
                result = Self::sleep_until(&config), if !check_cooldown => {
                    match result {
                        Ok(()) => {
                            self.run_tasks(&mut quit_notification).await;
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

    pub async fn sleep_until(config: &ScheduledTasksConfig) -> Result<(), ScheduledTaskError> {
        sleep_until_current_time_is_at(config.daily_start_time)
            .await
            .change_context(ScheduledTaskError::TimeError)?;
        Ok(())
    }

    pub async fn run_tasks(&self, quit_notification: &mut ServerQuitWatcher) {
        match self.run_tasks_and_return_result(quit_notification).await {
            Ok(()) => (),
            Err(e) => {
                error!("Some scheduled task failed, error: {:?}", e);
            }
        }
    }

    pub async fn run_tasks_and_return_result(
        &self,
        quit_notification: &mut ServerQuitWatcher,
    ) -> Result<(), ScheduledTaskError> {
        self.run_tasks_for_individual_accounts(quit_notification)
            .await?;
        self.run_tasks_for_logged_in_clients(quit_notification)
            .await?;
        self.save_profile_statistics().await?;
        self.delete_processed_reports_which_have_user_data().await?;
        backup_data(&self.state, quit_notification).await?;
        Ok(())
    }

    pub async fn save_profile_statistics(&self) -> Result<(), ScheduledTaskError> {
        let statistics = self
            .state
            .profile_statistics_cache()
            .update_statistics(self.state.read(), self.state.perf_counter_data_arc())
            .await
            .change_context(ScheduledTaskError::ProfileStatisticsError)?;

        db_write_raw!(self.state, move |cmds| {
            cmds.profile_admin_history()
                .save_profile_statistics(statistics)
                .await
        })
        .await
        .change_context(ScheduledTaskError::DatabaseError)?;

        Ok(())
    }

    pub async fn delete_processed_reports_which_have_user_data(
        &self,
    ) -> Result<(), ScheduledTaskError> {
        let run_delete = |report_type, deletion_wait_time| async move {
            db_write_raw!(self.state, move |cmds| {
                cmds.common()
                    .delete_processed_reports_if_needed(report_type, deletion_wait_time)
                    .await
            })
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;
            Result::Ok(())
        };

        let durations = self
            .state
            .config()
            .limits_common()
            .processed_report_deletion_wait_duration;
        run_delete(
            ReportTypeNumberInternal::ProfileName,
            durations.profile_name,
        )
        .await?;
        run_delete(
            ReportTypeNumberInternal::ProfileText,
            durations.profile_text,
        )
        .await?;
        Ok(())
    }

    pub async fn run_tasks_for_individual_accounts(
        &self,
        quit_notification: &mut ServerQuitWatcher,
    ) -> Result<(), ScheduledTaskError> {
        let accounts = self
            .state
            .read()
            .common()
            .account_ids_internal_vec()
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;

        let mut age_updated = 0;

        for id in accounts {
            if quit_notification.try_recv() != Err(TryRecvError::Empty) {
                return Err(ScheduledTaskError::QuitRequested.report());
            }

            let account = self
                .state
                .read()
                .common()
                .account(id)
                .await
                .change_context(ScheduledTaskError::DatabaseError)?;

            let account_state = account.state();

            if account_state != AccountState::InitialSetup {
                self.update_profile_age_if_needed(id, &mut age_updated)
                    .await?;
            }

            if account_state != AccountState::PendingDeletion {
                self.init_deletion_for_unused_account(id).await?;
            }

            if account_state == AccountState::PendingDeletion {
                self.delete_account_if_needed(id).await?;
            } else if account_state == AccountState::Banned {
                self.unban_account_if_needed(id).await?;
            }

            email::handle_email_notifications(&self.state, id)
                .await
                .change_context(ScheduledTaskError::DatabaseError)?;

            self.reset_api_limits(id).await?;
        }

        if age_updated != 0 {
            info!("Automatic profile age update count: {}", age_updated);
        }

        Ok(())
    }

    // TODO(optimize): This could run only when year changes
    pub async fn update_profile_age_if_needed(
        &self,
        id: AccountIdInternal,
        age_updated_count: &mut u64,
    ) -> Result<(), ScheduledTaskError> {
        let ages = self
            .state
            .read()
            .profile()
            .initial_profile_age(id)
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;

        let ages = if let Some(ages) = ages {
            ages
        } else {
            return Ok(());
        };

        let age_updated = db_write_raw!(self.state, move |cmds| {
            let profile = cmds.read().profile().profile(id).await?.profile;

            if ages.is_age_valid(profile.age) {
                // No update needed
                return Ok(false);
            }

            // Age update needed

            let age_plus_one = ProfileAge::new_clamped(profile.age.value() + 1);
            if age_plus_one == profile.age || !ages.is_age_valid(age_plus_one) {
                // Current profile age is 99 or incrementing age by one is not
                // enough.
                return Ok(false);
            }

            // Save the new age to database
            let profile_update = ProfileUpdateInternal {
                ptext: profile.ptext.clone(),
                name: profile.name.clone(),
                age: age_plus_one,
                attributes: profile
                    .attributes
                    .iter()
                    .cloned()
                    .map(|v| v.into())
                    .collect(),
            };
            let profile_update = profile_update
                .validate(
                    cmds.profile_attributes().schema(),
                    cmds.config().profile_name_regex(),
                    &profile,
                    None,
                    cmds.read().common().is_bot(id).await?,
                )
                .into_error_string(DataError::NotAllowed)?;
            cmds.profile().profile(id, profile_update).await?;

            cmds.events()
                .send_connected_event(id, EventToClientInternal::ProfileChanged)
                .await?;

            Ok(true)
        })
        .await
        .change_context(ScheduledTaskError::DatabaseError)?;

        if age_updated {
            *age_updated_count += 1;
        }

        Ok(())
    }

    pub async fn init_deletion_for_unused_account(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), ScheduledTaskError> {
        let last_seen_time = self
            .state
            .read()
            .profile()
            .last_seen_time_private(id)
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;

        // TODO(prod): When subscription feature is added prevent requesting
        //             deletion when subscription is active.

        if let Some(last_seen_time) = last_seen_time.last_seen_unix_time() {
            let inactive_account = last_seen_time.ut.add_seconds(
                self.state
                    .config()
                    .limits_account()
                    .init_deletion_for_inactive_accounts_wait_duration
                    .seconds,
            );
            if UnixTime::current_time().ut >= inactive_account.ut {
                db_write_raw!(self.state, move |cmds| {
                    cmds.account()
                        .delete()
                        .set_account_deletion_request_state(id, true)
                        .await
                })
                .await
                .change_context(ScheduledTaskError::DatabaseError)?;
            }
        }

        Ok(())
    }

    pub async fn delete_account_if_needed(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), ScheduledTaskError> {
        let deletion_allowed_time = self
            .state
            .read()
            .account()
            .delete()
            .account_deletion_state(id)
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;

        if let Some(deletion_allowed_time) = deletion_allowed_time.automatic_deletion_allowed {
            let current_time = UnixTime::current_time();
            if current_time.ut >= deletion_allowed_time.ut {
                db_write_raw!(self.state, move |cmds| {
                    cmds.account().delete().delete_account(id).await
                })
                .await
                .change_context(ScheduledTaskError::DatabaseError)?;
            }
        }

        Ok(())
    }

    pub async fn unban_account_if_needed(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), ScheduledTaskError> {
        let banned_until_time = self
            .state
            .read()
            .account()
            .ban()
            .ban_time(id)
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;

        if let Some(banned_until) = banned_until_time.banned_until
            && UnixTime::current_time().ut >= banned_until.ut
        {
            db_write_raw!(self.state, move |cmds| {
                cmds.account_admin()
                    .ban()
                    .set_account_ban_state(id, None, None, None, None)
                    .await
            })
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;
        }

        Ok(())
    }

    pub async fn reset_api_limits(&self, id: AccountIdInternal) -> Result<(), ScheduledTaskError> {
        self.state
            .api_limits(id)
            .reset_limits()
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;

        Ok(())
    }

    pub async fn run_tasks_for_logged_in_clients(
        &self,
        quit_notification: &mut ServerQuitWatcher,
    ) -> Result<(), ScheduledTaskError> {
        let accounts = self
            .state
            .read()
            .common()
            .account_ids_for_logged_in_clients()
            .await;

        for id in accounts {
            if quit_notification.try_recv() != Err(TryRecvError::Empty) {
                return Err(ScheduledTaskError::QuitRequested.report());
            }

            let last_seen_time = self
                .state
                .read()
                .profile()
                .last_seen_time_private(id)
                .await
                .change_context(ScheduledTaskError::DatabaseError)?;

            if let Some(last_seen_time) = last_seen_time.last_seen_unix_time() {
                let inactive_account = last_seen_time.ut.add_seconds(
                    self.state
                        .config()
                        .limits_account()
                        .inactivity_logout_wait_duration
                        .seconds,
                );
                if UnixTime::current_time().ut >= inactive_account.ut {
                    db_write_raw!(self.state, move |cmds| { cmds.common().logout(id).await })
                        .await
                        .change_context(ScheduledTaskError::DatabaseError)?;
                }
            }
        }

        Ok(())
    }
}
