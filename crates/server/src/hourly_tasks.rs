use std::time::Duration;

use server_api::{
    app::{ClientVersionTrackerProvider, GetConfig, WriteData},
    db_write_raw,
};
use server_common::result::{Result, WrappedResultExt};
use server_data::write::GetWriteCommandsCommon;
use server_data_account::write::GetWriteCommandsAccount;
use server_state::S;
use simple_backend::{app::PerfCounterDataProvider, ServerQuitWatcher};

use tokio::{task::JoinHandle, time::Instant};
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
pub enum HourlyTaskError {
    #[error("Sleep until next run of hourly tasks failed")]
    TimeError,

    #[error("Database update error")]
    DatabaseError,
}

#[derive(Debug)]
pub struct HourlyTaskManagerQuitHandle {
    task: JoinHandle<()>,
}

impl HourlyTaskManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("HourlyTaskManager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct HourlyTaskManager {
    state: S,
}

impl HourlyTaskManager {
    pub fn new_manager(
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> HourlyTaskManagerQuitHandle {
        let manager = Self { state };

        let task = tokio::spawn(manager.run(quit_notification));

        HourlyTaskManagerQuitHandle { task }
    }

    pub async fn run(self, mut quit_notification: ServerQuitWatcher) {
        const HOUR_IN_SECONDS: u64 = 60 * 60;
        let first_tick = Instant::now() + Duration::from_secs(HOUR_IN_SECONDS);
        let mut timer = tokio::time::interval_at(
            first_tick,
            Duration::from_secs(HOUR_IN_SECONDS),
        );

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    self.run_tasks().await;
                }
                _ = quit_notification.recv() => {
                    return;
                }
            }
        }
    }

    pub async fn run_tasks(&self) {
        match self.run_tasks_and_return_result().await {
            Ok(()) => (),
            Err(e) => {
                error!("Some hourly task failed, error: {:?}", e);
            }
        }
    }

    pub async fn run_tasks_and_return_result(
        &self,
    ) -> Result<(), HourlyTaskError> {
        self.save_performance_statistics().await?;
        if self.state.config().components().account {
            self.save_client_version_statistics().await?;
        }
        Ok(())
    }

    pub async fn save_performance_statistics(&self) -> Result<(), HourlyTaskError> {
        let statistics = self
            .state
            .perf_counter_data()
            .get_history_raw(true)
            .await;

        db_write_raw!(self.state, move |cmds| {
            cmds.common_history()
                .write_perf_data(statistics)
                .await
        })
        .await
        .change_context(HourlyTaskError::DatabaseError)?;

        Ok(())
    }

    pub async fn save_client_version_statistics(&self) -> Result<(), HourlyTaskError> {
        let statistics = self
            .state
            .client_version_tracker()
            .get_current_state_and_reset()
            .await;

        db_write_raw!(self.state, move |cmds| {
            cmds.account_admin_history()
                .save_client_version_statistics(statistics)
                .await
        })
        .await
        .change_context(HourlyTaskError::DatabaseError)?;

        Ok(())
    }
}
