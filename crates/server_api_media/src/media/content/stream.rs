use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

use super::quality::ContentSendingTracker;

/// Wraps a stream with a `ContentSendingTracker` guard.
///
/// The guard is held until the stream is fully consumed or dropped,
/// ensuring the concurrent sending counter stays elevated during the
/// entire response transmission.
pub struct ContentSendingStream<S> {
    inner: S,
    _guard: ContentSendingTracker,
}

impl<S> ContentSendingStream<S> {
    pub fn new(inner: S, _guard: ContentSendingTracker) -> Self {
        Self { inner, _guard }
    }
}

impl<S, T, E> Stream for ContentSendingStream<S>
where
    S: Stream<Item = Result<T, E>> + Unpin,
{
    type Item = Result<T, E>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
