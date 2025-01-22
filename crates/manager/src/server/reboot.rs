//! Handle automatic reboots

use std::{
    path::Path,
    time::Duration,
};

use error_stack::{Result, ResultExt};
use manager_config::Config;
use simple_backend_utils::time::sleep_until_current_time_is_at;
use tokio::{task::JoinHandle, time::sleep};
use tracing::{info, warn, error};

use super::{
    app::S, ServerQuitWatcher
};
use crate::{api::{GetConfig, GetScheduledTaskManager}, server::scheduled_task::ScheduledTaskManagerMessage};

/// If this file exists reboot system at some point. Works at least on Ubuntu.
const REBOOT_REQUIRED_PATH: &str = "/var/run/reboot-required";

#[derive(thiserror::Error, Debug)]
enum RebootError {
    #[error("Time related error")]
    Time,

    #[error("Config related error")]
    Config,

    #[error("Scheduled task error")]
    ScheduledTask,
}

#[derive(Debug)]
pub struct RebootManagerQuitHandle {
    task: JoinHandle<()>,
}

impl RebootManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("Reboot manager quit failed. Error: {:?}", e);
            }
        }
    }
}

pub struct RebootManager {
    state: S,
}

impl RebootManager {
    pub fn new_manager(
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> RebootManagerQuitHandle {
        let manager = Self {
            state,
        };

        let task = tokio::spawn(manager.run(quit_notification));

        RebootManagerQuitHandle {
            task,
        }
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        info!(
            "Automatic reboot status: {}",
            self.state.config().automatic_system_reboot().is_some()
        );

        let mut check_cooldown = false;

        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(120)), if check_cooldown => {
                    check_cooldown = false;
                }
                result = Self::sleep_until_reboot_check(self.state.config()), if !check_cooldown => {
                    match result {
                        Ok(()) => {
                            self.schedule_reboot_if_needed().await;
                        },
                        Err(e) => {
                            warn!("Sleep until reboot check failed. Error: {:?}", e);
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

    async fn schedule_reboot_if_needed(&self) {
        if Path::new(REBOOT_REQUIRED_PATH).exists() {
            info!("Reboot required file exists. Scheduling system reboot.");
            let notify_backend = self.state.config()
                .automatic_system_reboot()
                .map(|v| v.notify_backend)
                .unwrap_or_default();
            let result = self.state
                .scheduled_task_manager()
                .send_message(ScheduledTaskManagerMessage::ScheduleSystemReboot { notify_backend })
                .await
                .change_context(RebootError::ScheduledTask);

            if let Err(e) = result {
                error!("Reboot scheduling failed: {:?}", e);
            }
        } else {
            info!("No reboot needed");
        }
    }

    async fn sleep_until_reboot_check(config: &Config) -> Result<(), RebootError> {
        if let Some(reboot) = config.automatic_system_reboot() {
            sleep_until_current_time_is_at(reboot.scheduling_time)
                .await
                .change_context(RebootError::Time)
        } else {
            futures::future::pending::<()>().await;
            Err(RebootError::Config.into())
        }
    }
}
