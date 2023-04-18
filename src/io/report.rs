use crate::crawl::{CrawledNode, Crawler};
use crate::stats::{Database, DbReader};

use base64::{engine::general_purpose::STANDARD, Engine};
use mc_consensus_scp::{QuorumSet as McQuorumSet, QuorumSetMember};
use mc_crypto_keys::Ed25519Public;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
#[allow(unused_imports)] // necessary for tests
use std::collections::HashSet;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
/// Representation of a crawl::CrawledNode node in stellarbeat.io format.
/// The MobcoinFbas is a collection of MobcoinNodes.
pub struct MobcoinNode {
    #[serde(serialize_with = "key_to_base64")]
    pub public_key: Ed25519Public,
    pub hostname: String,
    pub port: u16,
    pub active: bool,
    pub quorum_set: QuorumSet,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub isp: String,
    pub geo_data: GeoData,
    pub latest_block: u64,
    pub ledger_version: u32,
    pub minimum_fee: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoData {
    pub country_name: String,
}

/// A MobcoinNode/ CrawledNode's QSet.
/// It is equivalent to a mc_consensus_scp::QuorumSet, just encoded differently.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumSet {
    pub threshold: u64,
    /// Validators are identified using their base64 encoded PKs
    pub validators: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inner_quorum_sets: Vec<QuorumSet>,
}

/// The MobileCoin FBAS.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
pub struct MobcoinFbas(Vec<MobcoinNode>);

/// The CrawlReport contains the timestamp, crawl duration, number of nodes (and number of
/// reachable nodes) as well as the MobcoinFbas.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlReport {
    /// The crawl's timestamp
    pub timestamp: String,
    /// How long the crawl took
    pub duration: Duration,
    /// The MobileCoin Nodes
    pub node_info: NodeInfo,
    pub nodes: MobcoinFbas,
    pub latest_block: LatestBlockInfo,
    pub minimum_fee: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LatestBlockInfo {
    #[default]
    None,
    Consensus(u64),
    Err(String),
}

/// Holds (general) data about the crawl and is included in the CrawlReport.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub total_nodes: usize,
    pub reachable_nodes: usize,
}

impl MobcoinFbas {
    pub fn create_mobcoin_fbas(crawler: &Crawler) -> Self {
        let nodes = crawler
            .mobcoin_nodes
            .iter()
            .map(|node| MobcoinNode::from_crawled_node(node.clone()))
            .collect();
        Self(nodes)
    }
}

impl CrawlReport {
    fn determine_minimum_fee(crawler: &Crawler) -> u64 {
        let mut min_fee = None;
        for node in &crawler.mobcoin_nodes {
            match min_fee {
                None => {
                    min_fee = Some(node.minimum_fee);
                }
                Some(mf) => {
                    if mf != node.minimum_fee {
                        return 0;
                    }
                }
            }
        }
        min_fee.unwrap_or(0)
    }

    fn determine_network_block_height(crawler: &Crawler) -> LatestBlockInfo {
        // reverse search in a hash map (value -> key)
        fn find_key_for_value(map: &HashMap<u64, u64>, value: u64) -> Option<u64> {
            map.iter()
                .find_map(|(key, val)| if *val == value { Some(*key) } else { None })
        }
        // Map<latest_block, count_of_nodes which_proclaim_it>
        let mut map = HashMap::<u64, u64>::new();
        for node in &crawler.mobcoin_nodes {
            *map.entry(node.latest_block).or_insert(0) += 1;
        }
        if map.is_empty() {
            return LatestBlockInfo::Err("no nodes found.".to_string());
        }

        let mut amount: Vec<u64> = map.values().cloned().collect::<Vec<u64>>();
        amount.sort_unstable_by(|a, b| b.cmp(a)); // reverse sorting

        // 2 different block lengths have the same number of »supporter nodes«
        // actually no decision possible; let's rely on our trusted bootstrap nodes
        if amount.len() > 1 && amount[0] == amount[1] {
            let bootstrap_peers = crawler
                .clone()
                .bootstrap_peers
                .into_iter()
                .collect::<Vec<String>>();

            let mut trusted_block = None;
            for node in &crawler.mobcoin_nodes {
                for bsp in &bootstrap_peers {
                    let (domain, port) = CrawledNode::fragment_mc_url(bsp.clone());
                    // is this node one of our trusted ones?
                    if (node.domain == domain) && (node.port == port) {
                        // Bäääm!
                        match trusted_block {
                            Some(trusted_block) => {
                                if trusted_block != node.latest_block {
                                    return LatestBlockInfo::Err(
                                        "nodes did not consent to a latest block because trusted nodes are discordant.".to_string(),
                                    );
                                }
                            }
                            _ => {
                                trusted_block = Some(node.latest_block);
                            }
                        };
                    }
                }
            }
            match trusted_block {
                Some(trusted_block) => {
                    return LatestBlockInfo::Consensus(trusted_block);
                }
                _ => {
                    return LatestBlockInfo::Err(
                        "nodes did not consent to a latest block because a unexpected error occured.".to_string(),
                    );
                }
            };
        }
        LatestBlockInfo::Consensus(find_key_for_value(&map, amount[0]).unwrap())
    }
    pub fn create_crawl_report(fbas: MobcoinFbas, crawler: &Crawler) -> Self {
        Self {
            timestamp: crawler.crawl_time.clone(),
            duration: crawler.crawl_duration,
            node_info: NodeInfo {
                total_nodes: fbas.0.len(),
                reachable_nodes: crawler.reachable_nodes,
            },
            nodes: fbas,
            latest_block: CrawlReport::determine_network_block_height(crawler),
            minimum_fee: CrawlReport::determine_minimum_fee(crawler),
        }
    }
}

impl QuorumSet {
    /// Converts a MobileCoin encoded QuorumSet to a Stellarbeat encoded QuorumSet
    fn from_mc_quorum_set(mc_quorum_set: McQuorumSet) -> Self {
        let threshold = mc_quorum_set.threshold.into();
        let mut validators: Vec<String> = Vec::new();
        let mut inner_quorum_sets: Vec<QuorumSet> = Vec::new();
        for member in mc_quorum_set.members.iter() {
            match member {
                QuorumSetMember::Node(node) => {
                    validators.push(STANDARD.encode(node.public_key));
                }
                QuorumSetMember::InnerSet(qs) => {
                    inner_quorum_sets.push(Self::from_mc_quorum_set(qs.clone()));
                }
            }
        }
        QuorumSet {
            threshold,
            validators,
            inner_quorum_sets,
        }
    }
}

impl MobcoinNode {
    fn from_crawled_node(crawled_node: CrawledNode) -> Self {
        let quorum_set = QuorumSet::from_mc_quorum_set(crawled_node.clone().quorum_set);
        let ip_addr = crawled_node.resolve_hostname_to_ip();
        let isp = DbReader::new(Database::Asn).lookup_isp(ip_addr);
        let country_name = DbReader::new(Database::Country).lookup_country(ip_addr);
        Self {
            public_key: crawled_node.public_key,
            hostname: crawled_node.domain,
            port: crawled_node.port,
            active: crawled_node.online,
            quorum_set,
            isp,
            geo_data: GeoData { country_name },
            latest_block: crawled_node.latest_block,
            ledger_version: crawled_node.network_block_version,
            minimum_fee: crawled_node.minimum_fee,
        }
    }
}

/// Serializes `buffer` to a lowercase hex string.
pub fn key_to_base64<T, S>(buffer: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: Serializer,
{
    serializer.serialize_str(&STANDARD.encode(&buffer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_consensus_scp::test_utils::test_node_id;

    #[test]
    fn mc_qset_without_inner_to_sbeat_qset() {
        let node_0 = test_node_id(0);
        let node_1 = test_node_id(1);
        let mc_qset = McQuorumSet::new(
            2,
            vec![
                QuorumSetMember::Node(node_0.clone()),
                QuorumSetMember::Node(node_1.clone()),
            ],
        );
        let validators = vec![
            STANDARD.encode(node_0.public_key),
            STANDARD.encode(node_1.public_key),
        ];
        let inner_quorum_sets = vec![];
        let expected = QuorumSet {
            threshold: 2,
            validators,
            inner_quorum_sets,
        };
        let actual = QuorumSet::from_mc_quorum_set(mc_qset.clone());
        assert!(mc_qset.is_valid());
        assert_eq!(expected, actual);
    }

    #[test]
    fn mc_qset_with_inner_to_sbeat_qset() {
        let node_0 = test_node_id(0);
        let node_1 = test_node_id(1);
        let node_2 = test_node_id(2);
        let node_3 = test_node_id(3);
        let node_4 = test_node_id(4);
        let mc_qset = McQuorumSet::new(
            2,
            vec![
                QuorumSetMember::Node(node_0.clone()),
                QuorumSetMember::Node(node_1.clone()),
                QuorumSetMember::InnerSet(McQuorumSet::new(
                    2,
                    vec![
                        QuorumSetMember::Node(node_2.clone()),
                        QuorumSetMember::Node(node_3.clone()),
                    ],
                )),
                QuorumSetMember::InnerSet(McQuorumSet::new(
                    1,
                    vec![QuorumSetMember::Node(node_4.clone())],
                )),
            ],
        );
        let validators = vec![
            STANDARD.encode(node_0.public_key),
            STANDARD.encode(node_1.public_key),
        ];
        let inner_quorum_sets = vec![
            QuorumSet {
                threshold: 2,
                validators: vec![
                    STANDARD.encode(node_2.public_key),
                    STANDARD.encode(node_3.public_key),
                ],
                inner_quorum_sets: Vec::default(),
            },
            QuorumSet {
                threshold: 1,
                validators: vec![STANDARD.encode(node_4.public_key)],
                inner_quorum_sets: Vec::default(),
            },
        ];
        let expected = QuorumSet {
            threshold: 2,
            validators,
            inner_quorum_sets,
        };
        let actual = QuorumSet::from_mc_quorum_set(mc_qset.clone());
        assert!(mc_qset.is_valid());
        assert_eq!(expected, actual);
    }

    #[test]
    fn crawled_node_to_mobcoin_node() {
        let node_0 = test_node_id(0);
        let node_1 = test_node_id(1);
        let crawled_node = CrawledNode {
            public_key: Ed25519Public::default(),
            domain: "test.foo.com".to_string(),
            port: 443,
            quorum_set: McQuorumSet::new(
                2,
                vec![
                    QuorumSetMember::Node(node_0.clone()),
                    QuorumSetMember::Node(node_1.clone()),
                ],
            ),
            online: false,
            latest_block: 4242,
            network_block_version: 42,
        };
        let quorum_set = QuorumSet::from_mc_quorum_set(crawled_node.quorum_set.clone());
        let expected = MobcoinNode {
            public_key: Ed25519Public::default(),
            hostname: "test.foo.com".to_string(),
            port: 443,
            quorum_set,
            active: false,
            isp: String::from(""),
            geo_data: GeoData {
                country_name: String::from("United States"),
            },
            latest_block: 4242,
            ledger_version: 42,
        };
        let actual = MobcoinNode::from_crawled_node(crawled_node);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_determine_network_block_height_failure() {
        let to_crawl: HashSet<String> = vec![
            "mc://node1.trusted.com:123".to_string(),
            "mc://node2.trusted.com:123".to_string(),
        ]
        .into_iter()
        .collect();
        let mut cnl = HashSet::<CrawledNode>::new();
        for i in 1..3 {
            let crawled_node = CrawledNode {
                public_key: Ed25519Public::default(),
                domain: format!("node{}.trusted.com", i),
                port: 123,
                quorum_set: McQuorumSet::new(0, vec![]),
                online: false,
                latest_block: i,
                network_block_version: 42,
            };
            cnl.insert(crawled_node);
        }
        let crawler = Crawler {
            bootstrap_peers: to_crawl.clone(),
            mobcoin_nodes: cnl.clone(),
            to_crawl,
            crawled: HashSet::new(),
            reachable_nodes: 2,
            crawl_duration: Duration::default(),
            crawl_time: String::default(),
        };
        let result = CrawlReport::determine_network_block_height(&crawler);
        assert!(
            matches!(result, LatestBlockInfo::Err(_)),
            "result is type of {:#?} because of {:#?}.",
            result,
            cnl
        );
    }
    #[test]
    fn test_determine_network_block_height() {
        let to_crawl: HashSet<String> = vec!["mc://node1.coins.com:123".to_string()]
            .into_iter()
            .collect();
        let mut cnl = HashSet::<CrawledNode>::new();
        for i in 1..4 {
            let crawled_node = CrawledNode {
                public_key: Ed25519Public::default(),
                domain: format!("node{}.coins.com", i),
                port: 123,
                quorum_set: McQuorumSet::new(0, vec![]),
                online: false,
                latest_block: i,
                network_block_version: 42,
            };
            cnl.insert(crawled_node);
        }
        let crawler = Crawler {
            bootstrap_peers: to_crawl.clone(),
            mobcoin_nodes: cnl.clone(),
            to_crawl,
            crawled: HashSet::new(),
            reachable_nodes: 2,
            crawl_duration: Duration::default(),
            crawl_time: String::default(),
        };
        let result = CrawlReport::determine_network_block_height(&crawler);
        assert!(
            matches!(result, LatestBlockInfo::Consensus(_)),
            "result is type of {:#?} because of {:#?}.",
            result,
            cnl
        );
    }
}
