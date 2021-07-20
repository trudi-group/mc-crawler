use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use url::Url;

use mc_consensus_scp::QuorumSet;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MobcoinNode {
    pub hostname: String,
    pub domain: String,
    pub port: u16,
    pub quorum_set: QuorumSet,
    pub online: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Crawler {
    pub mobcoin_nodes: HashSet<MobcoinNode>,
    pub to_crawl: Arc<RwLock<HashSet<String>>>,
    pub crawled: HashSet<String>,
    pub crawl_duration: Duration,
}

impl MobcoinNode {
    pub fn new(hostname: String, online: bool, quorum_set: QuorumSet) -> Self {
        let (domain, port) = Self::resolve_hostname(hostname.clone());
        MobcoinNode {
            hostname,
            domain,
            port,
            quorum_set,
            online,
        }
    }

    fn resolve_hostname(hostname: String) -> (String, u16) {
        let url = Url::parse(&hostname).expect("Failed to parse into Url");
        let domain = url.domain();
        let port = url.port();

        let (ip, port_nr) = if domain.is_none() || port.is_none() {
            (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).to_string(), 0)
        } else {
            if let Some(resolved) = domain {
                (String::from(resolved), port.unwrap_or_else(|| 0))
            } else {
                (IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)).to_string(), 0)
            }
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
            to_crawl: Arc::new(RwLock::new(to_crawl)),
            crawled: HashSet::new(),
            crawl_duration: Duration::default(),
        }
    }

    /// 0. Add he reporting node to the set of crawled nodes
    /// 1. Add node to the set to discovered nodes
    /// 2. Iterate over all members of the Qset and them to the set of peers that should be crawled
    pub fn handle_discovered_node(
        clone: Arc<Mutex<Crawler>>,
        crawled_node: String,
        node: MobcoinNode,
    ) {
        let mut crawler = clone.lock().expect("Mutex poisoned");
        crawler.crawled.insert(crawled_node.clone());
        crawler.mobcoin_nodes.insert(node.clone());
        let mut to_crawl_set = crawler.to_crawl.write().unwrap();
        for member in node.quorum_set.nodes() {
            let address = format!("{}{}", "mc://", member.responder_id.to_string());
            if crawler.crawled.get(&address).is_some() {
                debug!("Already crawled. {}", &member.responder_id.to_string());
                continue;
            } else {
                debug!("Adding {} to crawl queue.", address);
                to_crawl_set.insert(address);
            }
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

    /*#[test]
    fn create_new_crawler() {
        let bs_peer = "foo";
        let mut to_crawl: HashSet<String> = HashSet::new();
        to_crawl.insert(String::from("foo"));
        let expected = Crawler {
            mobcoin_nodes: HashSet::new(),
            to_crawl: Arc::new(RwLock::new(to_crawl)),
            crawled: HashSet::new(),
            crawl_duration: Duration::default(),
        };
        let actual = Crawler::new(bs_peer);
        assert_eq!(expected, actual);
    }*/
}
