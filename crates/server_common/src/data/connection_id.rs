use std::{fmt, net::IpAddr};

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

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}
