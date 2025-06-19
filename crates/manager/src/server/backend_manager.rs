//! Start and stop backend

use std::process::ExitStatus;

use error_stack::{Result, ResultExt};
use manager_config::file::ControlBackendConfig;
use tokio::{
    process::Command,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tracing::{error, info, warn};

use super::{ServerQuitWatcher, app::S};
use crate::api::GetConfig;

#[derive(thiserror::Error, Debug)]
pub enum BackendManagerError {
    #[error("Process start failed")]
    ProcessStartFailed,

    #[error("Process wait failed")]
    ProcessWaitFailed,

    #[error("Process stdin writing failed")]
    ProcessStdinFailed,

    #[error("Command failed with exit status: {0}")]
    CommandFailed(ExitStatus),

    #[error("Broken channel")]
    BrokenChannel,
}

#[derive(Debug)]
pub enum BackendManagerMessage {
    StartBackend { wait_start: oneshot::Sender<()> },
    StopBackend { wait_stop: oneshot::Sender<()> },
}

impl BackendManagerMessage {
    fn message_handled_info_sender(self) -> oneshot::Sender<()> {
        match self {
            BackendManagerMessage::StartBackend { wait_start } => wait_start,
            BackendManagerMessage::StopBackend { wait_stop } => wait_stop,
        }
    }
}

#[derive(Debug)]
pub struct BackendManagerQuitHandle {
    task: JoinHandle<()>,
    // Make sure Receiver works until the manager quits.
    _sender: mpsc::Sender<BackendManagerMessage>,
}

impl BackendManagerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("Backend manager quit failed. Error: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendManagerHandle {
    sender: mpsc::Sender<BackendManagerMessage>,
}

impl BackendManagerHandle {
    /// Does not wait that the backend is started
    pub async fn trigger_start_backend(&self) -> Result<(), BackendManagerError> {
        let (wait_start, _) = oneshot::channel();
        self.sender
            .send(BackendManagerMessage::StartBackend { wait_start })
            .await
            .change_context(BackendManagerError::BrokenChannel)?;
        Ok(())
    }

    pub async fn start_backend(&self) -> Result<(), BackendManagerError> {
        let (wait_start, receiver) = oneshot::channel();
        self.sender
            .send(BackendManagerMessage::StartBackend { wait_start })
            .await
            .change_context(BackendManagerError::BrokenChannel)?;
        receiver
            .await
            .change_context(BackendManagerError::BrokenChannel)
    }

    pub async fn stop_backend(&self) -> Result<(), BackendManagerError> {
        let (wait_stop, receiver) = oneshot::channel();
        self.sender
            .send(BackendManagerMessage::StopBackend { wait_stop })
            .await
            .change_context(BackendManagerError::BrokenChannel)?;
        receiver
            .await
            .change_context(BackendManagerError::BrokenChannel)
    }
}

pub struct BackendManagerInternalState {
    sender: mpsc::Sender<BackendManagerMessage>,
    receiver: mpsc::Receiver<BackendManagerMessage>,
}

pub struct BackendManager {
    receiver: mpsc::Receiver<BackendManagerMessage>,
    state: S,
    backend_running: bool,
}

impl BackendManager {
    pub fn new_channel() -> (BackendManagerHandle, BackendManagerInternalState) {
        let (sender, receiver) = mpsc::channel(10);
        let handle = BackendManagerHandle {
            sender: sender.clone(),
        };
        let state = BackendManagerInternalState { sender, receiver };
        (handle, state)
    }

    pub fn new_manager(
        state: S,
        internal_state: BackendManagerInternalState,
        quit_notification: ServerQuitWatcher,
    ) -> BackendManagerQuitHandle {
        let quit_handle_sender = internal_state.sender.clone();
        let manager = Self {
            receiver: internal_state.receiver,
            state,
            backend_running: false,
        };

        let task = tokio::spawn(manager.run(quit_notification.resubscribe()));

        BackendManagerQuitHandle {
            task,
            _sender: quit_handle_sender,
        }
    }

    async fn run(mut self, mut quit_notification: ServerQuitWatcher) {
        tokio::select! {
            _ = self.handle_messages() => (),
            _ = quit_notification.recv() => (),
        }

        if self.backend_running {
            if let Some(config) = self.state.config().control_backend().cloned() {
                if let Err(e) = self.stop_backend(&config).await {
                    error!("Backend stopping failed. Error: {:?}", e);
                }
            }
        }
    }

    async fn handle_messages(&mut self) {
        loop {
            let message = self.receiver.recv().await;
            match message {
                Some(message) => {
                    if let Some(config) = self.state.config().control_backend().cloned() {
                        match self.handle_message(message, &config).await {
                            Ok(()) => (),
                            Err(e) => error!("Backend manager error: {:?}", e),
                        }
                    } else {
                        warn!(
                            "Ignoring backend manager message. Backend controlling is not configured."
                        );
                        let _ = message.message_handled_info_sender().send(());
                    }
                }
                None => {
                    warn!("Backend manager channel closed");
                    return;
                }
            }
        }
    }

    async fn handle_message(
        &mut self,
        message: BackendManagerMessage,
        config: &ControlBackendConfig,
    ) -> Result<(), BackendManagerError> {
        match message {
            BackendManagerMessage::StartBackend { wait_start } => {
                if !self.backend_running {
                    self.start_backend(config).await?;
                    self.backend_running = true;
                }
                let _ = wait_start.send(());
            }
            BackendManagerMessage::StopBackend { wait_stop } => {
                if self.backend_running {
                    self.stop_backend(config).await?;
                    self.backend_running = false;
                }
                let _ = wait_stop.send(());
            }
        }

        Ok(())
    }

    async fn start_backend(
        &self,
        config: &ControlBackendConfig,
    ) -> Result<(), BackendManagerError> {
        let script = self.state.config().script_locations().systemctl_access();

        if !script.exists() {
            warn!("Script for starting the backend does not exist");
            return Ok(());
        }

        let status = Command::new("sudo")
            .arg(script)
            .arg("start")
            .arg(&config.service)
            .status()
            .await
            .change_context(BackendManagerError::ProcessWaitFailed)?;

        if !status.success() {
            tracing::error!("Start backend failed with status: {:?}", status);
            return Err(BackendManagerError::CommandFailed(status).into());
        } else {
            info!("Backend started");
        }

        Ok(())
    }

    async fn stop_backend(&self, config: &ControlBackendConfig) -> Result<(), BackendManagerError> {
        let script = self.state.config().script_locations().systemctl_access();

        if !script.exists() {
            warn!("Script for stopping the backend does not exist");
            return Ok(());
        }

        let status = Command::new("sudo")
            .arg(script)
            .arg("stop")
            .arg(&config.service)
            .status()
            .await
            .change_context(BackendManagerError::ProcessWaitFailed)?;

        if !status.success() {
            tracing::error!("Stop backend failed with status: {:?}", status);
            return Err(BackendManagerError::CommandFailed(status).into());
        } else {
            info!("Backend stopped");
        }

        Ok(())
    }
}
