use std::{fmt, net::IpAddr};

use model::PortNumber;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectionId {
    /// IP address from [SocketAddr] or `X-Forwarded-For` HTTP header
    ip: IpAddr,
    /// Port number from [SocketAddr]
    port: u16,
}

impl ConnectionId {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn ip(&self) -> IpAddr {
        self.ip
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConnectionDetails {
    id: ConnectionId,
    /// Client's port number (not available if reverse proxy TLS termination is used)
    port: Option<PortNumber>,
}

impl ConnectionDetails {
    pub fn new(id: ConnectionId, port: Option<PortNumber>) -> Self {
        Self { id, port }
    }

    pub fn id(&self) -> ConnectionId {
        self.id
    }

    pub fn port(&self) -> Option<PortNumber> {
        self.port
    }
}

impl fmt::Display for ConnectionDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)
    }
}
