use tokio::sync::{mpsc, broadcast, watch};

pub type WsQuitReady = mpsc::Receiver<()>;

/// Drop this when quit starts
pub type ServerQuitHandle = broadcast::Sender<()>;


/// Handle to WebSocket connections. Server main loop should use this
/// when closing the server.
#[derive(Debug)]
pub struct WebSocketManager {
    /// This must be dropped, so that the server quits.
    pub quit_handle: mpsc::Sender<()>,

    /// If this disconnects, the server quit is happening.
    pub server_quit_watcher: broadcast::Receiver<()>,
}

impl Clone for WebSocketManager {
    fn clone(&self) -> Self {
        Self { quit_handle: self.quit_handle.clone(), server_quit_watcher: self.server_quit_watcher.resubscribe() }
    }
}

impl WebSocketManager {
    pub fn new() -> (Self, WsQuitReady, ServerQuitHandle) {
        let (sender, receiver) = mpsc::channel(1);
        let (server_quit_handle, server_quit_watcher) = broadcast::channel(1);
        (Self { quit_handle: sender, server_quit_watcher }, receiver, server_quit_handle)
    }
}
