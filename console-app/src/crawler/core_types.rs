use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::time::Duration;

use mc_consensus_scp::QuorumSet;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MobcoinNode {
    hostname: String,
    ip_address: String,
    port: u16,
    public_key: String,
    quorum_set: QuorumSet,
    online: bool,
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Crawler {
    discovered_nodes: HashSet<MobcoinNode>,
    to_crawl: HashSet<MobcoinNode>,
    crawl_duration: Duration,
}

impl MobcoinNode {
    pub fn new(hostname: String) -> Self {
        let (ip_address, port) = Self::resolve_hostname(hostname.clone());
        MobcoinNode {
            hostname,
            ip_address,
            port,
            public_key: String::default(),
            quorum_set: QuorumSet::empty(),
            online: false,
        }
    }

    fn resolve_hostname(hostname: String) -> (String, u16) {
        let addr_iter = match hostname.to_socket_addrs() {
            Ok(addrs) => Some(addrs),
            Err(_) => None,
        };
        if addr_iter.is_some() {
            let (ip, port) = if let Some(resolved) = addr_iter.unwrap().next() {
                (resolved.ip().to_string(), resolved.port())
            } else {
                (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).to_string(), 0)
            };
            (ip, port)
        } else {
            (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).to_string(), 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_hostname_to_ip_port() {
        let hostname = "foo:443";
        let expected = (String::from("127.0.0.1"), 0);
        let actual = MobcoinNode::resolve_hostname(String::from(hostname));
        assert_eq!(expected, actual);
    }
}
