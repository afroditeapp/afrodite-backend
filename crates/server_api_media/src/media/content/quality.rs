use std::sync::atomic::{AtomicU64, Ordering};

use axum::http::{HeaderName, HeaderValue};
use headers::Header;
use model::ContentQualityVariant;

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

#[derive(Debug, Clone)]
pub struct ContentQualityHeader(pub ContentQualityVariant);

impl Header for ContentQualityHeader {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("q");
        &NAME
    }

    fn decode<'i, I>(_values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        Err(headers::Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let v = match self.0 {
            ContentQualityVariant::High => "h",
            ContentQualityVariant::Medium => "m",
            ContentQualityVariant::Low => "l",
        };
        if let Ok(value) = HeaderValue::from_str(v) {
            values.extend(std::iter::once(value));
        }
    }
}
