use std::sync::OnceLock;

use tokio::sync::{Mutex, mpsc};

use crate::ServerQuitWatcher;

fn get_quit_lock() -> &'static Mutex<Option<mpsc::Sender<()>>> {
    /// Use static for storing the ongoing websocket connection
    /// detection as storing the Sender in WebSocketManager causes
    /// ongoing HTTP connections to  prevent the server from shutting down.
    static QUIT_LOCK: OnceLock<Mutex<Option<mpsc::Sender<()>>>> = OnceLock::new();
    QUIT_LOCK.get_or_init(|| Mutex::new(None))
}

#[derive(Debug)]
pub struct WebSocketManager {
    /// If this disconnects, the server quit is happening.
    server_quit_watcher: ServerQuitWatcher,
}

impl Clone for WebSocketManager {
    fn clone(&self) -> Self {
        Self {
            server_quit_watcher: self.server_quit_watcher.resubscribe(),
        }
    }
}

impl WebSocketManager {
    pub async fn new(server_quit_watcher: ServerQuitWatcher) -> (Self, WsWatcher) {
        let (quit_lock, quit_handle) = mpsc::channel(1);
        *get_quit_lock().lock().await = Some(quit_lock);

        (
            Self {
                server_quit_watcher,
            },
            WsWatcher { quit_handle },
        )
    }

    /// Get the ongoing websocket connection detection lock or None if
    /// server is closing.
    pub async fn get_ongoing_ws_connection_quit_lock(&self) -> Option<mpsc::Sender<()>> {
        let quit_lock_storage = get_quit_lock().lock().await;
        quit_lock_storage.clone()
    }

    pub async fn server_quit_detected(&mut self) {
        let _ = self.server_quit_watcher.recv().await;
    }
}

pub struct WsWatcher {
    quit_handle: mpsc::Receiver<()>,
}

impl WsWatcher {
    pub async fn wait_for_quit(&mut self) {
        let mut quit_lock_storage = get_quit_lock().lock().await;
        let quit_lock = quit_lock_storage.take();
        drop(quit_lock);
        drop(quit_lock_storage);

        loop {
            match self.quit_handle.recv().await {
                Some(_) => (),
                None => break,
            }
        }
    }
}
