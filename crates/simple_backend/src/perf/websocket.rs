use std::sync::atomic::{AtomicU32, Ordering};


static WEBSOCKET_CONNECTION_COUNT: AtomicU32 = AtomicU32::new(0);

pub struct WebSocketConnectionTracker(());

impl WebSocketConnectionTracker {
    pub fn create() -> Self {
        WEBSOCKET_CONNECTION_COUNT.fetch_add(1, Ordering::Relaxed);
        Self(())
    }

    pub fn connection_count() -> u32 {
        WEBSOCKET_CONNECTION_COUNT.load(Ordering::Relaxed)
    }
}

impl Drop for WebSocketConnectionTracker {
    fn drop(&mut self) {
        WEBSOCKET_CONNECTION_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}
