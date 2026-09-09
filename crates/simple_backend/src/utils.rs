use std::net::{IpAddr, SocketAddr};

use axum::http::HeaderMap;
use tokio::sync::oneshot;

/// Sender only used for quit request message sending.
pub type QuitSender = oneshot::Sender<()>;

/// Receiver only used for quit request message receiving.
pub type QuitReceiver = oneshot::Receiver<()>;

/// Returns the client IP address from the `X-Forwarded-For` header if
/// present, otherwise falls back to the actual TCP connection IP address.
/// The rightmost IP address in the `X-Forwarded-For` header is used, as it
/// is the one added by the trusted reverse proxy closest to the server.
pub fn client_ip_from_http_header_if_possible(
    headers: &HeaderMap,
    connect_info: SocketAddr,
) -> IpAddr {
    const X_FORWARDED_FOR: &str = "x-forwarded-for";
    headers
        .get(X_FORWARDED_FOR)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.rsplit(',').next())
        .map(str::trim)
        .and_then(|v| v.parse::<IpAddr>().ok())
        .unwrap_or(connect_info.ip())
}
