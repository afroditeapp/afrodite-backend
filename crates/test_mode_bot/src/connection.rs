use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use api_client::models::EventToClient;
use error_stack::Result;
use test_mode_utils::client::TestError;
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub fn create_event_channel(
    enable_event_sending: Arc<AtomicBool>,
) -> (
    EventSenderAndQuitWatcher,
    EventReceiver,
    broadcast::Sender<()>,
) {
    let (event_sender, event_receiver) = mpsc::unbounded_channel();
    let (quit_handle, quit_watcher) = broadcast::channel(1);
    (
        EventSenderAndQuitWatcher {
            event_sender: EventSender {
                enable_event_sending,
                event_sender,
            },
            quit_watcher,
        },
        EventReceiver { event_receiver },
        quit_handle,
    )
}

#[derive(Debug, Clone)]
pub struct EventSender {
    enable_event_sending: Arc<AtomicBool>,
    event_sender: mpsc::UnboundedSender<EventToClient>,
}

impl EventSender {
    pub async fn send_if_sending_enabled(&self, event: EventToClient) {
        if self.enable_event_sending.load(Ordering::Relaxed) {
            let _ = self.event_sender.send(event);
        }
    }
}

#[derive(Debug)]
pub struct EventSenderAndQuitWatcher {
    pub event_sender: EventSender,
    pub quit_watcher: broadcast::Receiver<()>,
}

impl Clone for EventSenderAndQuitWatcher {
    fn clone(&self) -> Self {
        Self {
            event_sender: self.event_sender.clone(),
            quit_watcher: self.quit_watcher.resubscribe(),
        }
    }
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

#[derive(Debug, Default)]
pub struct BotConnections {
    /// Setting this true will enable sending the connection
    /// events to event channel.
    pub enable_event_sending: Arc<AtomicBool>,
    connections: Option<ApiConnection>,
    events: Option<EventReceiver>,
}

impl BotConnections {
    pub fn set_connections(&mut self, connections: ApiConnection) {
        self.connections = Some(connections);
    }

    pub fn set_events(&mut self, events: EventReceiver) {
        self.events = Some(events);
    }

    pub fn unwrap_account_connections(&mut self) -> ApiConnection {
        self.connections
            .take()
            .expect("Account connections are missing")
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
