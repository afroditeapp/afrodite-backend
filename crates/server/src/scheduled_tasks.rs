use std::time::Duration;
use model_profile::{AccountIdInternal, AccountState, EventToClientInternal, ProfileAge, ProfileUpdate, ProfileUpdateInternal};
use server_api::{db_write_raw, result::WrappedContextExt, DataError};
use server_common::result::{Result, WrappedResultExt};
use server_data::read::GetReadCommandsCommon;
use server_data_account::read::GetReadCommandsAccount;
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile, app::ProfileStatisticsCacheProvider};
use server_state::S;
use simple_backend::{utils::time::sleep_until_current_time_is_at, ServerQuitWatcher};
use simple_backend_config::file::ScheduledTasksConfig;
use simple_backend_utils::IntoReportFromString;
use tokio::{sync::broadcast::error::TryRecvError, task::JoinHandle, time::sleep};
use tracing::{error, info, warn};
use server_api::app::{GetConfig, ReadData, WriteData};

#[derive(thiserror::Error, Debug)]
pub enum ScheduledTaskError {
    #[error("Sleep until next run of scheduled tasks failed")]
    TimeError,

    #[error("Database update error")]
    DatabaseError,

    #[error("Profile statistics error")]
    ProfileStatisticsError,

    #[error("Unexpected server quit request detected while scheduled tasks were running")]
    QuitRequested,
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

    pub async fn run(
        self,
        mut quit_notification: ServerQuitWatcher,
    ) {
        let mut check_cooldown = false;
        let config = self.state.config().simple_backend().scheduled_tasks();

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
        sleep_until_current_time_is_at(config.daily_run_time)
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

    pub async fn run_tasks_and_return_result(&self, quit_notification: &mut ServerQuitWatcher) -> Result<(), ScheduledTaskError> {
        self.run_tasks_for_individual_accounts(quit_notification).await?;
        self.save_profile_statistics().await?;
        // TODO(prod): SQLite database backups
        Ok(())
    }

    pub async fn save_profile_statistics(&self) -> Result<(), ScheduledTaskError> {
        let statistics = self
            .state
            .profile_statistics_cache()
            .update_statistics(&self.state)
            .await
            .change_context(ScheduledTaskError::ProfileStatisticsError)?;

        db_write_raw!(self.state, move |cmds| {
            cmds
                .profile_admin_history()
                .save_profile_statistics(statistics)
                .await
        })
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;

        Ok(())
    }

    pub async fn run_tasks_for_individual_accounts(&self, quit_notification: &mut ServerQuitWatcher) -> Result<(), ScheduledTaskError> {
        let accounts = self.state
            .read()
            .account()
            .account_ids_internal_vec()
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;

        let mut age_updated = 0;

        for id in accounts {
            if quit_notification.try_recv() != Err(TryRecvError::Empty) {
                return Err(ScheduledTaskError::QuitRequested.report())
            }

            let account = self.state
                .read()
                .common()
                .account(id)
                .await
                .change_context(ScheduledTaskError::DatabaseError)?;

            if account.state() != AccountState::InitialSetup {
                self.update_profile_age_if_needed(id, &mut age_updated).await?;
            }
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
        let ages = self.state
            .read()
            .profile()
            .accepted_profile_ages(id)
            .await
            .change_context(ScheduledTaskError::DatabaseError)?;

        let ages = if let Some(ages) = ages {
            ages
        } else {
            return Ok(())
        };

        let age_updated = db_write_raw!(self.state, move |cmds| {
            let profile = cmds
                .read()
                .profile()
                .profile(id)
                .await?
                .profile;

            if ages.is_age_valid(profile.age) {
                // No update needed
                return Ok(false);
            }

            // Age update needed

            let age_plus_one = ProfileAge::new_clamped(profile.age.value() + 1);
            if age_plus_one == profile.age || !ages.is_age_valid(age_plus_one)  {
                // Current profile age is 99 or incrementing age by one is not
                // enough.
                return Ok(false);
            }

            // Save the new age to database
            let profile_update = ProfileUpdate {
                ptext: profile.ptext.clone(),
                name: profile.name.clone(),
                age: age_plus_one,
                attributes: profile.attributes
                    .iter()
                    .cloned()
                    .map(|v| v.into())
                    .collect(),
            };
            let profile_update = profile_update
                .validate(cmds.config().profile_attributes(), &profile, None)
                .into_error_string(DataError::NotAllowed)?;
            let profile_update = ProfileUpdateInternal::new(profile_update);
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
}
