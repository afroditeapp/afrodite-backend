
use error_stack::{ResultExt, Result};
use manager_model::BackupMessage;
use tokio::{sync::{mpsc, oneshot}, task::JoinHandle};
use tracing::{warn, error};

use crate::{api::utils::BackupLinkClient, server::ServerQuitWatcher};

#[derive(Debug)]
pub struct BackupLinkConnectionReceiver {
    pub receiver: mpsc::Receiver<BackupMessage>,
}

#[derive(Debug)]
pub struct BackupLinkConnectionSender {
    pub sender: mpsc::Sender<BackupMessage>,
}

#[derive(Debug)]
pub enum BackupLinkManagerMessage {
    ReplaceTargetConnection {
        handle_sender: oneshot::Sender<BackupLinkConnectionReceiver>,
    },
    ReplaceSourceConnection {
        handle_sender: oneshot::Sender<Option<BackupLinkConnectionReceiver>>,
    },
    CleanConnection {
        client_type: BackupLinkClient,
    },
    ReceiveMessage {
        client_type: BackupLinkClient,
        message: BackupMessage,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum BackupLinkError {
    #[error("Broken channel")]
    BrokenChannel,

    #[error("Broken link")]
    BrokenLink,

    #[error("Timeout")]
    Timeout,

    #[error("Serialize")]
    Serialize,

    #[error("Deserialize")]
    Deserialize,
}

#[derive(Debug)]
pub struct BackupLinkManagerServerQuitHandle {
    task: JoinHandle<()>,
    // Make sure Receiver works until the manager quits.
    _sender: mpsc::Sender<BackupLinkManagerMessage>,
}

impl BackupLinkManagerServerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("Backup link server manager quit failed. Error: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackupLinkManagerHandleServer {
    sender: mpsc::Sender<BackupLinkManagerMessage>,
}

impl BackupLinkManagerHandleServer {
    pub async fn replace_target_connection(&self) -> Result<BackupLinkConnectionReceiver, BackupLinkError> {
        let (handle_sender, handle_receiver) = oneshot::channel();
        self.sender
            .send(BackupLinkManagerMessage::ReplaceTargetConnection { handle_sender })
            .await
            .change_context(BackupLinkError::BrokenChannel)?;
        handle_receiver
            .await
            .change_context(BackupLinkError::BrokenChannel)
    }

    /// None is returned when target client is not connected.
    pub async fn replace_source_connection(&self) -> Result<Option<BackupLinkConnectionReceiver>, BackupLinkError> {
        let (handle_sender, handle_receiver) = oneshot::channel();
        self.sender
            .send(BackupLinkManagerMessage::ReplaceSourceConnection { handle_sender })
            .await
            .change_context(BackupLinkError::BrokenChannel)?;
        handle_receiver
            .await
            .change_context(BackupLinkError::BrokenChannel)
    }

    pub async fn clean_connection(&self, client_type: BackupLinkClient) -> Result<(), BackupLinkError> {
        self.sender
            .send(BackupLinkManagerMessage::CleanConnection { client_type })
            .await
            .change_context(BackupLinkError::BrokenChannel)
    }

    pub async fn receive_message(&self, client_type: BackupLinkClient, message: BackupMessage) -> Result<(), BackupLinkError> {
        self.sender
            .send(BackupLinkManagerMessage::ReceiveMessage { client_type, message })
            .await
            .change_context(BackupLinkError::BrokenChannel)
    }
}

pub struct BackupLinkManagerInternalState {
    sender: mpsc::Sender<BackupLinkManagerMessage>,
    receiver: mpsc::Receiver<BackupLinkManagerMessage>,
}

pub struct BackupLinkManagerServer {
    receiver: mpsc::Receiver<BackupLinkManagerMessage>,
    connection_target: Option<BackupLinkConnectionSender>,
    connection_source: Option<BackupLinkConnectionSender>,
}

impl BackupLinkManagerServer {
    pub fn new_channel() -> (BackupLinkManagerHandleServer, BackupLinkManagerInternalState) {
        let (sender, receiver) = mpsc::channel(10);
        let handle = BackupLinkManagerHandleServer {
            sender: sender.clone(),
        };
        let state = BackupLinkManagerInternalState {
            sender,
            receiver,
        };
        (handle, state)
    }

    pub fn new_manager(
        internal_state: BackupLinkManagerInternalState,
        quit_notification: ServerQuitWatcher,
    ) -> BackupLinkManagerServerQuitHandle {
        let quit_handle_sender = internal_state.sender.clone();
        let manager = Self {
            receiver: internal_state.receiver,
            connection_source: None,
            connection_target: None,
        };

        let task = tokio::spawn(manager.run(quit_notification.resubscribe()));

        BackupLinkManagerServerQuitHandle {
            task,
            _sender: quit_handle_sender,
        }
    }

    async fn run(self, mut quit_notification: ServerQuitWatcher) {
        tokio::select! {
            _ = self.handle_messages() => (),
            _ = quit_notification.recv() => (),
        }
    }

    async fn handle_messages(
        mut self,
    ) {
        loop {
            let message = self.receiver.recv().await;
            match message {
                Some(message) => {
                    self.handle_message(message).await;
                }
                None => {
                    warn!("JsonRcpLink manager channel closed");
                    return;
                }
            }
        }
    }

    async fn handle_message(
        &mut self,
        message: BackupLinkManagerMessage,
    ) {
        match message {
            BackupLinkManagerMessage::ReplaceSourceConnection { handle_sender } => {
                if self.connection_target.is_some() {
                    let (sender, receiver) = mpsc::channel(10);
                    self.connection_source = Some(BackupLinkConnectionSender { sender });
                    let _ = handle_sender.send(Some(BackupLinkConnectionReceiver {
                        receiver,
                    }));
                } else {
                    let _ = handle_sender.send(None);
                }
            },
            BackupLinkManagerMessage::ReplaceTargetConnection { handle_sender } => {
                let (sender, receiver) = mpsc::channel(10);
                self.connection_target = Some(BackupLinkConnectionSender { sender });
                self.connection_source = None;
                let _ = handle_sender.send(BackupLinkConnectionReceiver {
                    receiver,
                });
            },
            BackupLinkManagerMessage::CleanConnection { client_type } => {
                if let Some(connection) = &self.connection(client_type) {
                    if connection.sender.is_closed() {
                        *self.connection(client_type) = None;
                    }
                }
            }
            BackupLinkManagerMessage::ReceiveMessage { client_type, message } => {
                let next_location = match client_type {
                    BackupLinkClient::Source => &mut self.connection_target,
                    BackupLinkClient::Target => &mut self.connection_source,
                };

                if let Some(sender) = next_location {
                    match sender.sender.send(message).await {
                        Ok(()) => (),
                        Err(_) => (),
                    }
                }
            },
        }
    }

    pub fn connection(&mut self, client_type: BackupLinkClient) -> &mut Option<BackupLinkConnectionSender> {
        match client_type {
            BackupLinkClient::Source => &mut self.connection_source,
            BackupLinkClient::Target => &mut self.connection_target,
        }
    }
}
