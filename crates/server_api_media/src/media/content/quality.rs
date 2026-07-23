use std::sync::atomic::{AtomicU64, Ordering};

static CONTENT_SENDING_COUNT: AtomicU64 = AtomicU64::new(0);

pub struct ContentSendingTracker(());

impl ContentSendingTracker {
    pub fn track() -> Self {
        CONTENT_SENDING_COUNT.fetch_add(1, Ordering::Relaxed);
        Self(())
    }

    pub fn concurrent_count() -> u64 {
        CONTENT_SENDING_COUNT.load(Ordering::Relaxed)
    }
}

impl Drop for ContentSendingTracker {
    fn drop(&mut self) {
        CONTENT_SENDING_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}
