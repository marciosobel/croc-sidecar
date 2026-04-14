use std::net::IpAddr;

/// A representation of a relay for `croc`.
#[derive(Debug, Clone, PartialEq)]
pub struct Relay {
    /// The IP address of the relay.
    pub address: IpAddr,
    /// The port of the relay.
    pub port: u16,
}

impl Relay {
    /// Creates a new `Relay`.
    pub fn new<Ip: Into<IpAddr>>(address: Ip, port: u16) -> Self {
        Self {
            address: address.into(),
            port,
        }
    }
}

impl std::fmt::Display for Relay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.address {
            IpAddr::V4(address) => write!(f, "{}:{}", address, self.port),
            IpAddr::V6(address) => write!(f, "[{}]:{}", address, self.port),
        }
    }
}
