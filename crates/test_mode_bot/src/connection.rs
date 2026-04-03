use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use error_stack::Result;
use test_mode_utils::{client::TestError, websocket_protocol::EventToClient};
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub fn create_event_channel(
    enable_event_sending: EventInfoHandle,
) -> (
    EventSenderAndQuitWatcher,
    EventReceiver,
    ClientMessageSender,
    broadcast::Sender<()>,
) {
    let (event_sender, event_receiver) = mpsc::unbounded_channel();
    let (client_message_sender, client_message_receiver) = mpsc::unbounded_channel();
    let (quit_handle, quit_watcher) = broadcast::channel(1);
    (
        EventSenderAndQuitWatcher {
            event_sender: EventSender {
                event_info_handle: enable_event_sending,
                event_sender,
            },
            client_message_receiver: ClientMessageReceiver {
                client_message_receiver,
            },
            quit_watcher,
        },
        EventReceiver { event_receiver },
        ClientMessageSender {
            client_message_sender,
        },
        quit_handle,
    )
}

#[derive(Debug, Clone)]
pub struct EventSender {
    event_info_handle: EventInfoHandle,
    event_sender: mpsc::UnboundedSender<EventToClient>,
}

impl EventSender {
    pub async fn send_if_sending_enabled(&self, event: EventToClient) {
        if event.should_be_forwarded_when_events_disabled()
            || self.event_info_handle.are_events_enabled()
        {
            let _ = self.event_sender.send(event);
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientMessageSender {
    client_message_sender: mpsc::UnboundedSender<Vec<u8>>,
}

impl ClientMessageSender {
    pub fn send(&self, message: Vec<u8>) -> Result<(), TestError> {
        self.client_message_sender
            .send(message)
            .map_err(|_| TestError::ClientMessageChannelClosed.report())
    }
}

#[derive(Debug)]
pub struct ClientMessageReceiver {
    client_message_receiver: mpsc::UnboundedReceiver<Vec<u8>>,
}

impl ClientMessageReceiver {
    pub async fn recv(&mut self) -> Option<Vec<u8>> {
        self.client_message_receiver.recv().await
    }
}

#[derive(Debug)]
pub struct EventSenderAndQuitWatcher {
    pub event_sender: EventSender,
    pub client_message_receiver: ClientMessageReceiver,
    pub quit_watcher: broadcast::Receiver<()>,
}

#[derive(Debug)]
pub struct EventReceiver {
    event_receiver: mpsc::UnboundedReceiver<EventToClient>,
}

impl EventReceiver {
    pub async fn recv(&mut self) -> Option<EventToClient> {
        self.event_receiver.recv().await
    }
}

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct WsConnection {
    task: JoinHandle<()>,
}

impl WsConnection {
    pub fn new(task: JoinHandle<()>) -> Self {
        Self { task }
    }

    /// Close EventReceiver before calling this.
    pub async fn close(self) {
        let _ = self.task.await;
    }
}

#[derive(Debug)]
pub struct ApiConnection {
    pub connection: Option<WsConnection>,
    /// Drop this to close all WebSockets
    pub quit_handle: broadcast::Sender<()>,
}

impl ApiConnection {
    pub async fn close(mut self) {
        drop(self.quit_handle);
        if let Some(account) = self.connection.take() {
            let _ = account.close().await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventInfoHandle {
    enable_event_sending: Arc<AtomicBool>,
}

impl EventInfoHandle {
    pub fn are_events_enabled(&self) -> bool {
        self.enable_event_sending.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Default)]
pub struct BotConnections {
    /// Setting this true will enable sending the connection
    /// events to event channel.
    enable_event_sending: Arc<AtomicBool>,
    connections: Option<ApiConnection>,
    events: Option<EventReceiver>,
    client_message_sender: Option<ClientMessageSender>,
}

impl BotConnections {
    pub fn set_connections(&mut self, connections: ApiConnection) {
        self.connections = Some(connections);
    }

    pub fn set_events(&mut self, events: EventReceiver) {
        self.events = Some(events);
    }

    pub fn set_client_message_sender(&mut self, sender: ClientMessageSender) {
        self.client_message_sender = Some(sender);
    }

    pub fn event_info_handle(&self) -> EventInfoHandle {
        EventInfoHandle {
            enable_event_sending: self.enable_event_sending.clone(),
        }
    }

    pub fn are_events_enabled(&self) -> bool {
        self.enable_event_sending.load(Ordering::Relaxed)
    }

    pub fn enable_events(&self) {
        self.enable_event_sending.store(true, Ordering::Relaxed);
    }

    pub fn disable_events(&self) {
        self.enable_event_sending.store(true, Ordering::Relaxed);
    }

    pub fn unwrap_account_connections(&mut self) -> ApiConnection {
        self.connections
            .take()
            .expect("Account connections are missing")
    }

    pub fn send_client_message(&self, message: Vec<u8>) -> Result<(), TestError> {
        let sender = self
            .client_message_sender
            .as_ref()
            .ok_or(TestError::ClientMessageSendingHandleMissing.report())?;
        sender.send(message)
    }

    /// Receive next event without timeout.
    pub async fn recv_event(&mut self) -> Result<EventToClient, TestError> {
        if !self.enable_event_sending.load(Ordering::Relaxed) {
            return Err(TestError::EventReceivingHandleDisabled.report());
        }

        self.recv_event_internal().await
    }

    /// Receive next event without requiring global event forwarding.
    ///
    /// Can be used for receiving events which return true value from
    /// [EventToClient::should_be_forwarded_when_events_disabled].
    pub async fn recv_event_unchecked(&mut self) -> Result<EventToClient, TestError> {
        self.recv_event_internal().await
    }

    async fn recv_event_internal(&mut self) -> Result<EventToClient, TestError> {
        let events = self
            .events
            .as_mut()
            .ok_or(TestError::EventReceivingHandleMissing.report())?;

        events
            .recv()
            .await
            .ok_or(TestError::EventChannelClosed.report())
    }

    /// Wait event if event sending is enabled or timeout after 5 seconds
    pub async fn wait_event(
        &mut self,
        check: impl Fn(&EventToClient) -> bool,
    ) -> Result<(), TestError> {
        if !self.enable_event_sending.load(Ordering::Relaxed) {
            return Ok(());
        }

        let events = self
            .events
            .as_mut()
            .ok_or(TestError::EventReceivingHandleMissing.report())?;

        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => Err(TestError::EventReceivingTimeout.report()),
            event_or_error = wait_until_specific_event(check, events) => event_or_error,
        }
    }
}

async fn wait_until_specific_event(
    check: impl Fn(&EventToClient) -> bool,
    events: &mut EventReceiver,
) -> Result<(), TestError> {
    loop {
        let event = events
            .recv()
            .await
            .ok_or(TestError::EventChannelClosed.report())?;
        if check(&event) {
            return Ok(());
        }
    }
}
