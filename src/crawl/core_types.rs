use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use url::Url;

use mc_consensus_scp::QuorumSet as McQuorumSet;
use mc_crypto_keys::Ed25519Public;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct CrawledNode {
    pub public_key: Ed25519Public,
    pub domain: String,
    pub port: u16,
    pub quorum_set: McQuorumSet,
    pub online: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Crawler {
    pub mobcoin_nodes: HashSet<CrawledNode>,
    pub to_crawl: HashSet<String>,
    pub crawled: HashSet<String>,
    pub crawl_duration: Duration,
    pub crawl_time: String,
}

impl CrawledNode {
    pub fn new(url: String, online: bool, quorum_set: McQuorumSet) -> Self {
        let (domain, port) = Self::resolve_url(url);
        CrawledNode {
            public_key: Ed25519Public::default(),
            domain,
            port,
            quorum_set,
            online,
        }
    }

    /// Return 0.0.0.0 as an address if not resolvable otherwise the stats functions would return one own's geolocation
    fn resolve_url(url: String) -> (String, u16) {
        let url = Url::parse(&url).expect("Failed to parse into Url");
        let domain = url.domain();
        let port = url.port();

        let (ip, port_nr) = if domain.is_none() || port.is_none() {
            (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)).to_string(), 0)
        } else if let Some(resolved) = domain {
            (String::from(resolved), port.unwrap_or(0))
        } else {
            (IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)).to_string(), 0)
        };
        (ip, port_nr)
    }
}

impl Crawler {
    pub fn new(bootstrap_peer: &str) -> Self {
        let mut to_crawl: HashSet<String> = HashSet::new();
        to_crawl.insert(String::from(bootstrap_peer));
        Crawler {
            mobcoin_nodes: HashSet::new(),
            to_crawl,
            crawled: HashSet::new(),
            crawl_duration: Duration::default(),
            crawl_time: String::default(),
        }
    }

    /// 0. Add he reporting node to the set of crawled nodes
    /// 1. Add node to the set to discovered nodes
    /// 2. Iterate over all members of the Qset and them to the set of peers that should be crawled
    /// 3. Get the crawled node's PK from the response and add
    pub fn handle_discovered_node(&mut self, crawled_node: String, node: &mut CrawledNode) {
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
    pub fn get_public_keys_from_quorum_sets(&self) -> HashSet<CrawledNode> {
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
        let actual = CrawledNode::resolve_url(String::from(url));
        assert_eq!(expected, actual);
    }

    #[test]
    fn create_new_crawler() {
        let bs_peer = "foo";
        let mut to_crawl: HashSet<String> = HashSet::new();
        to_crawl.insert(String::from("foo"));
        let expected = Crawler {
            mobcoin_nodes: HashSet::new(),
            to_crawl: to_crawl,
            crawled: HashSet::new(),
            crawl_duration: Duration::default(),
            crawl_time: String::default(),
        };
        let actual = Crawler::new(bs_peer);
        assert_eq!(expected, actual);
    }
}
