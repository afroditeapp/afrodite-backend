use std::{fmt::Debug, net::{IpAddr, Ipv4Addr}};

use error_stack::{Result, ResultExt};
use ipnet::{IpAddrRange, IpNet, Ipv4AddrRange};
use simple_backend_utils::ContextExt;

use crate::{file::IpListConfig, GetConfigError};

#[derive(Clone)]
pub struct IpList {
    name: String,
    networks: Vec<IpNet>,
    ranges: Vec<IpAddrRange>,
}

impl Debug for IpList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IpList")
    }
}

impl IpList {
    pub(crate) fn new(config: &IpListConfig) -> Result<Self, GetConfigError> {
        let file = std::fs::read_to_string(&config.file)
            .change_context(GetConfigError::LoadConfig)?;

        let mut networks = vec![];
        let mut ranges = vec![];

        for l in file.lines() {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }

            if l.contains('/') {
                let network = l
                    .parse::<IpNet>()
                    .change_context(GetConfigError::LoadFileError)?;
                networks.push(network);
            } else if l.contains('-') {
                let mut iter = l.split('-');
                let first = iter
                    .next()
                    .ok_or(GetConfigError::LoadFileError.report())
                    .attach_printable_lazy(|| format!("Invalid IP range: {}", l))?;
                let second = iter
                    .next()
                    .ok_or(GetConfigError::LoadFileError.report())
                    .attach_printable_lazy(|| format!("Invalid IP range: {}", l))?;
                let range: IpAddrRange = Ipv4AddrRange::new(
                    first.parse::<Ipv4Addr>().change_context(GetConfigError::LoadFileError)?,
                    second.parse::<Ipv4Addr>().change_context(GetConfigError::LoadFileError)?,
                ).into();
                ranges.push(range);
            } else {
                let address = l.parse::<IpAddr>().change_context(GetConfigError::LoadFileError)?;
                networks.push(address.into());
            }
        }

        Ok(Self {
            name: config.name.clone(),
            networks: IpNet::aggregate(&networks),
            ranges,
        })
    }

    pub fn contains(&self, address: IpAddr) -> bool {
        // TODO(optimize, low priority): Use more efficient algorithm

        for n in &self.networks {
            if n.contains(&address) {
                return true;
            }
        }

        for r in &self.ranges {
            if r.into_iter().any(|v| v == address) {
                return true;
            }
        }

        false
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
