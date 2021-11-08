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
        let mut addrs = match hostname.to_socket_addrs() {
            Ok(socket) => socket,
            Err(e) => {
                warn!("Error resolving address {}: {}", e, hostname);
                Vec::default().into_iter()
            }
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

    /// 0. Add the reporting node to the set of crawled nodes
    /// 1. Add node to the set to discovered nodes
    /// 2. Iterate over all members of the Qset and add them to the set of peers that should be crawled
    pub(crate) fn handle_discovered_node(&mut self, crawled_node: String, node: &mut CrawledNode) {
        self.to_crawl.remove(&crawled_node);
        self.crawled.insert(crawled_node);
        self.mobcoin_nodes.insert(node.clone());
        for member in node.quorum_set.nodes() {
            let address = format!("{}{}", "mc://", member.responder_id.to_string());
            if self.crawled.get(&address).is_some() {
                continue;
            } else {
                debug!("Adding {} to crawl queue.", address);
                self.to_crawl.insert(address.clone());
            }
        }
    }

    /// Looks for each node's PK in the other node's Qsets
    pub(crate) fn get_public_keys_from_quorum_sets(&self) -> HashSet<CrawledNode> {
        let mut mobcoin_nodes_with_pks: HashSet<CrawledNode> = HashSet::new();
        // First get each node's PK
        for node in self.mobcoin_nodes.iter() {
            let responder_id = format!("{}{}:{}", "mc://", node.domain, node.port);
            for other_node in self.mobcoin_nodes.iter() {
                if other_node != node {
                    for member in other_node.quorum_set.nodes() {
                        let address = format!("{}{}", "mc://", member.responder_id.to_string());
                        if node.public_key == Ed25519Public::default() && responder_id == address {
                            let node_now_with_pk = CrawledNode {
                                public_key: member.public_key,
                                ..node.clone()
                            };
                            mobcoin_nodes_with_pks.insert(node_now_with_pk);
                            break;
                        }
                    }
                }
            }
        }
        mobcoin_nodes_with_pks
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
            to_crawl: to_crawl,
            crawled: HashSet::new(),
            reachable_nodes: 0,
            crawl_duration: Duration::default(),
            crawl_time: String::default(),
        };
        let actual = Crawler::new(bs_peers);
        assert_eq!(expected, actual);
    }
}
