use std::{collections::HashMap, num::Wrapping, time::Duration};

use error_stack::{Result, ResultExt};
use manager_model::{
    JsonRpcLinkHeader, JsonRpcLinkMessage, JsonRpcLinkMessageType, JsonRpcRequest, JsonRpcResponse,
};
use simple_backend_utils::ContextExt;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tracing::{error, warn};

use crate::server::ServerQuitWatcher;

#[derive(Debug)]
pub struct JsonRpcLinkConnectionReceiver {
    pub receiver: mpsc::Receiver<JsonRpcLinkMessage>,
}

#[derive(Debug)]
pub struct JsonRcpLinkConnectionSender {
    pub sender: mpsc::Sender<JsonRpcLinkMessage>,
}

#[derive(Debug)]
pub enum JsonRcpLinkManagerMessage {
    ReplaceConnection {
        handle_sender: oneshot::Sender<JsonRpcLinkConnectionReceiver>,
    },
    CleanConnection,
    ReceiveMessage {
        message: JsonRpcLinkMessage,
    },
    DoRequest {
        message: String,
        sender: oneshot::Sender<String>,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum JsonRcpLinkError {
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
pub struct JsonRcpLinkManagerServerQuitHandle {
    task: JoinHandle<()>,
    // Make sure Receiver works until the manager quits.
    _sender: mpsc::Sender<JsonRcpLinkManagerMessage>,
}

impl JsonRcpLinkManagerServerQuitHandle {
    pub async fn wait_quit(self) {
        match self.task.await {
            Ok(()) => (),
            Err(e) => {
                warn!("JsonRcpLink server manager quit failed. Error: {:?}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsonRcpLinkManagerHandleServer {
    sender: mpsc::Sender<JsonRcpLinkManagerMessage>,
}

impl JsonRcpLinkManagerHandleServer {
    pub async fn replace_connection(
        &self,
    ) -> Result<JsonRpcLinkConnectionReceiver, JsonRcpLinkError> {
        let (handle_sender, handle_receiver) = oneshot::channel();
        self.sender
            .send(JsonRcpLinkManagerMessage::ReplaceConnection { handle_sender })
            .await
            .change_context(JsonRcpLinkError::BrokenChannel)?;
        handle_receiver
            .await
            .change_context(JsonRcpLinkError::BrokenChannel)
    }

    pub async fn clean_connection(&self) -> Result<(), JsonRcpLinkError> {
        self.sender
            .send(JsonRcpLinkManagerMessage::CleanConnection)
            .await
            .change_context(JsonRcpLinkError::BrokenChannel)
    }

    pub async fn receive_message(
        &self,
        message: JsonRpcLinkMessage,
    ) -> Result<(), JsonRcpLinkError> {
        self.sender
            .send(JsonRcpLinkManagerMessage::ReceiveMessage { message })
            .await
            .change_context(JsonRcpLinkError::BrokenChannel)
    }

    /// Timeouts in 10 seconds
    pub async fn do_request(
        &self,
        message: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, JsonRcpLinkError> {
        let message =
            serde_json::to_string(&message).change_context(JsonRcpLinkError::Serialize)?;

        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(JsonRcpLinkManagerMessage::DoRequest { message, sender })
            .await
            .change_context(JsonRcpLinkError::BrokenChannel)?;

        let r = tokio::select! {
            m = receiver =>
                m.change_context(JsonRcpLinkError::BrokenLink),
            _ = tokio::time::sleep(Duration::from_secs(10)) =>
                Err(JsonRcpLinkError::Timeout.report()),
        };

        let response = r?;

        serde_json::from_str(&response).change_context(JsonRcpLinkError::Deserialize)
    }
}

pub struct JsonRcpLinkManagerInternalState {
    sender: mpsc::Sender<JsonRcpLinkManagerMessage>,
    receiver: mpsc::Receiver<JsonRcpLinkManagerMessage>,
}

pub struct JsonRcpLinkManagerServer {
    receiver: mpsc::Receiver<JsonRcpLinkManagerMessage>,
    connection: Option<JsonRcpLinkConnectionSender>,
    next_sequence_number: Wrapping<u32>,
    waiting_response: HashMap<u32, oneshot::Sender<String>>,
}

impl JsonRcpLinkManagerServer {
    pub fn new_channel() -> (
        JsonRcpLinkManagerHandleServer,
        JsonRcpLinkManagerInternalState,
    ) {
        let (sender, receiver) = mpsc::channel(10);
        let handle = JsonRcpLinkManagerHandleServer {
            sender: sender.clone(),
        };
        let state = JsonRcpLinkManagerInternalState { sender, receiver };
        (handle, state)
    }

    pub fn new_manager(
        internal_state: JsonRcpLinkManagerInternalState,
        quit_notification: ServerQuitWatcher,
    ) -> JsonRcpLinkManagerServerQuitHandle {
        let quit_handle_sender = internal_state.sender.clone();
        let manager = Self {
            receiver: internal_state.receiver,
            connection: None,
            next_sequence_number: Wrapping(0),
            waiting_response: HashMap::new(),
        };

        let task = tokio::spawn(manager.run(quit_notification.resubscribe()));

        JsonRcpLinkManagerServerQuitHandle {
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

    async fn handle_messages(mut self) {
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

    async fn handle_message(&mut self, message: JsonRcpLinkManagerMessage) {
        match message {
            JsonRcpLinkManagerMessage::ReplaceConnection { handle_sender } => {
                let (sender, receiver) = mpsc::channel(10);
                self.connection = Some(JsonRcpLinkConnectionSender { sender });
                let _ = handle_sender.send(JsonRpcLinkConnectionReceiver { receiver });
                self.waiting_response.clear();
            }
            JsonRcpLinkManagerMessage::CleanConnection => {
                if let Some(connection) = &self.connection {
                    if connection.sender.is_closed() {
                        self.connection = None;
                        self.waiting_response.clear();
                    }
                }
            }
            JsonRcpLinkManagerMessage::ReceiveMessage { message } => {
                match message.header.message_type {
                    JsonRpcLinkMessageType::Empty => (),
                    JsonRpcLinkMessageType::ServerRequest => warn!(
                        "Ignoring ServerRequest message. Client should send ServerResponse messages."
                    ),
                    JsonRpcLinkMessageType::ServerResponse => {
                        let sequence_number = message.header.sequence_number.0;
                        if let Some(handle) = self.waiting_response.remove(&sequence_number) {
                            match handle.send(message.data) {
                                Ok(()) => (),
                                Err(_) => warn!(
                                    "Sending message to response wait handle failed, message sequence number {}",
                                    sequence_number
                                ),
                            }
                        } else {
                            warn!(
                                "Missing response wait handle, message sequence number {}",
                                sequence_number
                            );
                        }
                    }
                }
            }
            JsonRcpLinkManagerMessage::DoRequest { message, sender } => {
                let Some(connection) = &self.connection else {
                    return;
                };

                let sequence_number = self.next_sequence_number;
                let message = JsonRpcLinkMessage {
                    header: JsonRpcLinkHeader {
                        message_type: JsonRpcLinkMessageType::ServerRequest,
                        sequence_number,
                    },
                    data: message,
                };

                if connection.sender.send(message).await.is_err() {
                    return;
                }

                self.garbage_collect_waiting_responses();

                self.waiting_response.insert(sequence_number.0, sender);

                self.next_sequence_number += 1;
            }
        }
    }

    fn garbage_collect_waiting_responses(&mut self) {
        let mut to_be_deleted = vec![];
        for (sequence_number, sender) in &self.waiting_response {
            if sender.is_closed() {
                to_be_deleted.push(*sequence_number);
            }
        }

        for number in to_be_deleted {
            self.waiting_response.remove(&number);
        }
    }
}
