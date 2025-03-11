//! Handle automatic reboots

use std::{
    process::ExitStatus,
    sync::Arc,
};

use error_stack::{Result, ResultExt};
use manager_model::ManualTaskType;
use tokio::{process::Command, sync::mpsc, task::JoinHandle};
use tracing::{info, warn};

use super::{
    app::S, client::ApiManager, state::MountStateStorage, update::backend::reset_backend_data, ServerQuitWatcher
};
use crate::{api::{GetBackendManager, GetConfig}, server::mount::MountMode};

#[derive(thiserror::Error, Debug)]
pub enum TaskError {
    #[error("Channel broken")]
    ChannelBroken,

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

    #[error("Stop backend failed")]
    StopBackendFailed,

    #[error("Start backend failed")]
    StartBackendFailed,

    #[error("Backend utils")]
    BackendUtils,
}

#[derive(Debug)]
pub struct TaskManagerQuitHandle {
    task: JoinHandle<()>,
    // Make sure Receiver works until the manager quits.
    _sender: mpsc::Sender<ManualTaskType>,
}

impl TaskManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("Task manager quit failed. Error: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskManagerHandle {
    sender: mpsc::Sender<ManualTaskType>,
}

impl TaskManagerHandle {
    pub async fn send_message(&self, message: ManualTaskType) -> Result<(), TaskError> {
        self.sender
            .send(message)
            .await
            .change_context(TaskError::ChannelBroken)?;

        Ok(())
    }
}

pub struct TaskManagerInternalState {
    sender: mpsc::Sender<ManualTaskType>,
    receiver: mpsc::Receiver<ManualTaskType>,
}

pub struct TaskManager {
    receiver: mpsc::Receiver<ManualTaskType>,
    state: S,
    mount_state: Arc<MountStateStorage>,
}

impl TaskManager {
    pub fn new_channel() -> (TaskManagerHandle, TaskManagerInternalState) {
        let (sender, receiver) = mpsc::channel(1);
        let handle = TaskManagerHandle {
            sender: sender.clone(),
        };
        let state = TaskManagerInternalState {
            sender,
            receiver,
        };
        (handle, state)
    }

    pub fn new_manager(
        internal_state: TaskManagerInternalState,
        state: S,
        mount_state: Arc<MountStateStorage>,
        quit_notification: ServerQuitWatcher,
    ) -> TaskManagerQuitHandle {
        let quit_handle_sender = internal_state.sender.clone();
        let manager = Self {
            receiver: internal_state.receiver,
            state,
            mount_state,
        };

        let task = tokio::spawn(manager.run(quit_notification));

        TaskManagerQuitHandle {
            task,
            _sender: quit_handle_sender,
        }
    }

    pub async fn run(mut self, mut quit_notification: ServerQuitWatcher) {
        loop {
            tokio::select! {
                message = self.receiver.recv() => {
                    match message {
                        Some(message) => {
                            self.handle_message(message).await;
                        }
                        None => {
                            warn!("Reboot manager channel closed");
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

    pub async fn handle_message(&self, message: ManualTaskType) {
        let result = match message {
            ManualTaskType::SystemReboot =>
                self.run_reboot().await,
            ManualTaskType::BackendRestart =>
                self.backend_restart_and_optional_data_reset(false).await,
            ManualTaskType::BackendDataReset =>
                self.backend_restart_and_optional_data_reset(true).await,
        };

        match result {
            Ok(()) => {
                info!("Action {:?} completed", message);
            }
            Err(e) => {
                warn!("Action {:?} failed. Error: {:?}", message, e);
            }
        }
    }

    pub async fn run_reboot(&self) -> Result<(), TaskError> {
        match self.mount_state.get(|s| s.mount_state.mode()).await {
            MountMode::MountedWithRemoteKey => {
                info!("Remote encryption key detected. Checking encryption key availability before rebooting");
                self.api_manager()
                    .get_encryption_key()
                    .await
                    .change_context(TaskError::GetKeyFailed)?;
                info!("Remote encryption key is available");
            }
            _ => (),
        }

        info!("Rebooting system");

        if self.state.config().debug_mode() {
            warn!("Skipping reboot because debug mode is enabled");
            return Ok(());
        }

        let status = Command::new("sudo")
            .arg("reboot")
            .status()
            .await
            .change_context(TaskError::ProcessStartFailed)?;

        if !status.success() {
            return Err(TaskError::CommandFailed(status).into());
        }

        Ok(())
    }

    async fn backend_restart_and_optional_data_reset(
        &self,
        data_reset: bool,
    ) -> Result<(), TaskError> {
        self.state
            .backend_manager()
            .stop_backend()
            .await
            .change_context(TaskError::StopBackendFailed)?;

        if data_reset {
            if let Some(config) = self.state.config().manual_tasks_config().allow_backend_data_reset {
                reset_backend_data(&config.backend_data_dir)
                    .await
                    .change_context(TaskError::BackendUtils)?
            } else {
                warn!("Skipping backend data reset because it is not enabled from config file");
            }
        }

        self.state
            .backend_manager()
            .start_backend()
            .await
            .change_context(TaskError::StartBackendFailed)?;

        Ok(())
    }

    fn api_manager(&self) -> ApiManager<'_> {
        ApiManager::new(&self.state)
    }
}
