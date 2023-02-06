use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::time::Duration;
use url::Url;

use mc_consensus_scp::QuorumSet as McQuorumSet;
use mc_crypto_keys::Ed25519Public;

/// A CrawledNode is a MobileCoin network node that we have learned of during the crawl. The
/// Crawler keeps a tally of these during a crawl and each will later be transformed to a MobCoinNode.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) struct CrawledNode {
    pub(crate) public_key: Ed25519Public,
    pub(crate) domain: String,
    pub(crate) port: u16,
    pub(crate) quorum_set: McQuorumSet,
    pub(crate) online: bool,
}

/// The Crawler object steers a crawl.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Crawler {
    /// A HashSet of discovered nodes
    pub(crate) mobcoin_nodes: HashSet<CrawledNode>,
    /// A HashSet of nodes to be crawled
    pub(crate) to_crawl: HashSet<String>,
    /// A HashSet of nodes that have been crawled
    pub crawled: HashSet<String>,
    /// The number of nodes the crawler got a response from
    pub(crate) reachable_nodes: usize,
    /// How long the crawl took
    pub(crate) crawl_duration: Duration,
    /// The crawl's timestamp
    pub crawl_time: String,
}

impl CrawledNode {
    /// Create a new CrawledNode using its hostname, connectivity status and Qset.
    pub(crate) fn new(url: String, online: bool, quorum_set: McQuorumSet) -> Self {
        let (domain, port) = Self::fragment_mc_url(url);
        CrawledNode {
            public_key: Ed25519Public::default(),
            domain,
            port,
            quorum_set,
            online,
        }
    }

    /// Return 0.0.0.0 as an address if not resolvable otherwise the stats functions would return one own's geolocation
    fn fragment_mc_url(url: String) -> (String, u16) {
        let url = Url::parse(&url).expect("Failed to parse into Url");
        let domain = url.domain();
        let port = url.port();

        let (ip, port_nr) = if domain.is_none() || port.is_none() {
            (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)).to_string(), 0)
        } else if let Some(host) = domain {
            (String::from(host), port.unwrap_or(0))
        } else {
            (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)).to_string(), 0)
        };
        (ip, port_nr)
    }

    pub fn resolve_hostname_to_ip(&self) -> IpAddr {
        let hostname = format!("{}:{}", self.domain, self.port);
        let mut addrs = if let Ok(socket) = hostname.to_socket_addrs() {
            socket
        } else {
            warn!("Error resolving {hostname}");
            Vec::default().into_iter()
        };
        if let Some(resolved) = addrs.next() {
            resolved.ip()
        } else {
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
        }
    }
}

impl Crawler {
    /// Create a new Crawler and add bootstrap peers.
    pub fn new(bootstrap_peers: Vec<String>) -> Self {
        let mut to_crawl: HashSet<String> = HashSet::new();
        for peer in bootstrap_peers {
            to_crawl.insert(peer);
        }
        Crawler {
            mobcoin_nodes: HashSet::new(),
            to_crawl,
            crawled: HashSet::new(),
            reachable_nodes: 0,
            crawl_duration: Duration::default(),
            crawl_time: String::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_url_to_ip_port() {
        let url = "foo:443";
        let expected = (String::from("0.0.0.0"), 0);
        let actual = CrawledNode::fragment_mc_url(String::from(url));
        assert_eq!(expected, actual);
    }

    #[test]
    fn create_new_crawler() {
        let bs_peers = vec![String::from("foo"), String::from("bar")];
        let mut to_crawl: HashSet<String> = HashSet::new();
        to_crawl.insert(String::from("foo"));
        to_crawl.insert(String::from("bar"));
        let expected = Crawler {
            mobcoin_nodes: HashSet::new(),
            to_crawl,
            crawled: HashSet::new(),
            reachable_nodes: 0,
            crawl_duration: Duration::default(),
            crawl_time: String::default(),
        };
        let actual = Crawler::new(bs_peers);
        assert_eq!(expected, actual);
    }
}
