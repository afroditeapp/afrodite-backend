//! Handle automatic reboots

use std::{
    process::ExitStatus,
    sync::Arc, time::Duration,
};

use error_stack::{Result, ResultExt};
use futures::lock::Mutex;
use manager_config::file::ScheduledTasksConfig;
use manager_model::{MaintenanceTask, MaintenanceTime, ScheduledTaskStatus};
use simple_backend_model::UnixTime;
use simple_backend_utils::time::{seconds_until_current_time_is_at, sleep_until_current_time_is_at};
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};
use tracing::{info, warn};

use super::{
    app::S, task::TaskManagerMessage, ServerQuitWatcher
};
use crate::api::{GetConfig, GetTaskManager};


#[derive(thiserror::Error, Debug)]
pub enum ScheduledTaskError {
    #[error("Broken channel")]
    BrokenChannel,

    #[error("Time related error")]
    TimeError,

    #[error("Config related error")]
    ConfigError,

    #[error("Command failed with exit status: {0}")]
    CommandFailed(ExitStatus),

    #[error("Process start failed")]
    ProcessStartFailed,

    #[error("Process wait failed")]
    ProcessWaitFailed,

    #[error("Invalid output")]
    InvalidOutput,

    #[error("Getting encryption key failed")]
    GetKeyFailed,
}

#[derive(Debug)]
pub struct ScheduledTaskManagerQuitHandle {
    task: JoinHandle<()>,
    // Make sure Receiver works until the manager quits.
    _sender: mpsc::Sender<ScheduledTaskManagerMessage>,
}

impl ScheduledTaskManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("Scheduled task manager quit failed. Error: {:?}", e);
            }
        }
    }
}

#[derive(Debug)]
pub enum ScheduledTaskManagerMessage {
    ScheduleBackendRestart {
        notify_backend: bool,
    },
    ScheduleSystemReboot {
        notify_backend: bool,
    },
    UnscheduleBackendRestart,
    UnscheduleSystemReboot,
}

#[derive(Debug, Clone)]
pub struct ScheduledTaskManagerHandle {
    sender: mpsc::Sender<ScheduledTaskManagerMessage>,
    state: Arc<Mutex<ScheduledTaskStatus>>,
}

impl ScheduledTaskManagerHandle {
    pub async fn send_message(&self, message: ScheduledTaskManagerMessage) -> Result<(), ScheduledTaskError> {
        self.sender
            .send(message)
            .await
            .change_context(ScheduledTaskError::BrokenChannel)?;
        Ok(())
    }

    pub async fn status(&self) -> ScheduledTaskStatus {
        self.state.lock().await.clone()
    }

    pub async fn maintenance_time(&self) -> Option<MaintenanceTime> {
        let state = self.state.lock().await;
        let r = state.backend_restart
            .as_ref()
            .map(|v| v.time)
            .or(state.system_reboot.as_ref().map(|v| v.time));

        r.map(MaintenanceTime)
    }
}

pub struct ScheduledTaskManagerInternalState {
    sender: mpsc::Sender<ScheduledTaskManagerMessage>,
    receiver: mpsc::Receiver<ScheduledTaskManagerMessage>,
    state: Arc<Mutex<ScheduledTaskStatus>>,
}

pub struct ScheduledTaskManager {
    receiver: mpsc::Receiver<ScheduledTaskManagerMessage>,
    internal_state: Arc<Mutex<ScheduledTaskStatus>>,
    state: S,
}

impl ScheduledTaskManager {
    pub fn new_channel() -> (ScheduledTaskManagerHandle, ScheduledTaskManagerInternalState) {
        let (sender, receiver) = mpsc::channel(1);
        let state = Arc::new(Mutex::new(ScheduledTaskStatus::default()));
        let handle = ScheduledTaskManagerHandle {
            sender: sender.clone(),
            state: state.clone()
        };
        let state = ScheduledTaskManagerInternalState {
            sender,
            receiver,
            state,
        };
        (handle, state)
    }

    pub fn new_manager(
        internal_state: ScheduledTaskManagerInternalState,
        state: S,
        quit_notification: ServerQuitWatcher,
    ) -> ScheduledTaskManagerQuitHandle {
        let quit_handle_sender = internal_state.sender.clone();
        let manager = Self {
            internal_state: internal_state.state,
            receiver: internal_state.receiver,
            state,
        };

        let task = tokio::spawn(manager.run(quit_notification));

        ScheduledTaskManagerQuitHandle {
            task,
            _sender: quit_handle_sender,
        }
    }

    pub async fn run(self, quit_notification: ServerQuitWatcher) {
        if let Some(config) = self.state.config().scheduled_tasks().cloned() {
            self.run_enabled(quit_notification, config.clone()).await
        } else {
            self.run_disabled(quit_notification).await
        }
    }

    pub async fn run_disabled(mut self, mut quit_notification: ServerQuitWatcher) {
        loop {
            tokio::select! {
                message = self.receiver.recv() => {
                    match message {
                        Some(message) => {
                            warn!("Skipping message {:?}, scheduled tasks are disabled", message);
                        }
                        None => {
                            warn!("Scheduled task manager channel closed");
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

    pub async fn run_enabled(
        mut self,
        mut quit_notification: ServerQuitWatcher,
        config: ScheduledTasksConfig,
    ) {
        let manager = ScheduledTaksManagerInternal {
            config: config.clone(),
            internal_state: self.internal_state,
            state: self.state,
        };

        let mut check_cooldown = false;

        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(120)), if check_cooldown => {
                    check_cooldown = false;
                }
                result = Self::sleep_until_run_scheduled_tasks_check(&config), if !check_cooldown => {
                    match result {
                        Ok(()) => {
                            manager.run_scheduled_tasks().await;
                        },
                        Err(e) => {
                            warn!("Sleep until run scheduled tasks check failed. Error: {:?}", e);
                        }
                    }
                    check_cooldown = true;
                }
                message = self.receiver.recv() => {
                    match message {
                        Some(message) => {
                            manager.handle_message(message).await;
                        }
                        None => {
                            warn!("Scheduled task manager channel closed");
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

    async fn sleep_until_run_scheduled_tasks_check(config: &ScheduledTasksConfig) -> Result<(), ScheduledTaskError> {
        sleep_until_current_time_is_at(config.daily_start_time)
            .await
            .change_context(ScheduledTaskError::TimeError)
    }
}

struct ScheduledTaksManagerInternal {
    internal_state: Arc<Mutex<ScheduledTaskStatus>>,
    state: S,
    config: ScheduledTasksConfig,
}

impl ScheduledTaksManagerInternal {
    async fn run_scheduled_tasks(&self) {
        let mut state = self.internal_state.lock().await;
        let result = if state.backend_restart.is_some() {
            self.state
                .task_manager()
                .send_message(TaskManagerMessage::BackendRestart)
                .await
        } else if state.system_reboot.is_some() {
            self.state
                .task_manager()
                .send_message(TaskManagerMessage::SystemReboot)
                .await
        } else {
            return;
        };

        state.backend_restart = None;
        state.system_reboot = None;

        match result {
            Ok(()) => (),
            Err(e) => {
                warn!("Task running failed. Error: {:?}", e);
            }
        }
    }

    async fn handle_message(&self, message: ScheduledTaskManagerMessage) {
        let result = match message {
            ScheduledTaskManagerMessage::ScheduleBackendRestart { notify_backend } =>
                self.schedule_backend_restart(notify_backend).await,
            ScheduledTaskManagerMessage::ScheduleSystemReboot { notify_backend } =>
                self.schedule_system_reboot(notify_backend).await,
            ScheduledTaskManagerMessage::UnscheduleBackendRestart =>
                self.unschedule_backend_restart().await,
            ScheduledTaskManagerMessage::UnscheduleSystemReboot =>
                self.unschedule_system_reboot().await,
        };

        self.state.refresh_state_to_backend().await;

        match result {
            Ok(()) => {
                info!("Action {:?} completed", message);
            }
            Err(e) => {
                warn!("Action {:?} failed. Error: {:?}", message, e);
            }
        }
    }

    async fn schedule_backend_restart(&self, notify_backend: bool) -> Result<(), ScheduledTaskError> {
        if !self.config.allow_backend_restart {
            return Ok(());
        }

        self.internal_state.lock().await.backend_restart =
            Some(self.new_maintenance_task(notify_backend)?);
        Ok(())
    }

    async fn schedule_system_reboot(&self, notify_backend: bool) -> Result<(), ScheduledTaskError> {
        if !self.config.allow_system_reboot {
            return Ok(());
        }

        self.internal_state.lock().await.system_reboot =
            Some(self.new_maintenance_task(notify_backend)?);
        Ok(())
    }

    async fn unschedule_backend_restart(&self) -> Result<(), ScheduledTaskError> {
        self.internal_state.lock().await.backend_restart = None;
        Ok(())
    }

    async fn unschedule_system_reboot(&self) -> Result<(), ScheduledTaskError> {
        self.internal_state.lock().await.system_reboot = None;
        Ok(())
    }

    fn new_maintenance_task(&self, notify_backend: bool) -> Result<MaintenanceTask, ScheduledTaskError> {
        Ok(MaintenanceTask {
            time: self.task_start_time()?,
            notify_backend,
        })
    }

    fn task_start_time(&self) -> Result<UnixTime, ScheduledTaskError> {
        let seconds = seconds_until_current_time_is_at(self.config.daily_start_time)
            .change_context(ScheduledTaskError::TimeError)?;
        let seconds = TryInto::<u32>::try_into(seconds)
            .change_context(ScheduledTaskError::TimeError)?;
        Ok(UnixTime::current_time().add_seconds(seconds))
    }
}
