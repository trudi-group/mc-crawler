use crate::crawl::core_types::*;

use std::{collections::HashSet, str::FromStr, sync::Arc};

use grpcio::{ChannelBuilder, EnvBuilder};
use mc_common::logger;
use mc_consensus_api::{
    consensus_peer::GetLatestMsgResponse, consensus_peer_grpc::ConsensusPeerApiClient,
};
use mc_consensus_scp::QuorumSet;
use mc_crypto_keys::Ed25519Public;
use mc_peers::ConsensusMsg;
use mc_util_grpc::ConnectionUriGrpcioChannel;
use mc_util_serial::deserialize;
use mc_util_uri::ConsensusClientUri as ClientUri;

impl Crawler {
    /// Opens an RPC channel to the peer which can be used for communication later
    pub(crate) fn prepare_rpc(peer: String) -> Option<ConsensusPeerApiClient> {
        let env = Arc::new(EnvBuilder::new().build());
        let logger = logger::create_root_logger();
        let node_uri = match ClientUri::from_str(&peer) {
            Ok(uri) => Some(uri),
            Err(_) => {
                warn!("Error in Node URI: {}", peer);
                return None;
            }
        };
        let ch = ChannelBuilder::default_channel_builder(env)
            .connect_to_uri(&node_uri.unwrap(), &logger);
        let consensus_client = ConsensusPeerApiClient::new(ch);
        Some(consensus_client)
    }

    /// The bytes of the RPC response is deserialised into an McQuorumSet::QuorumSet
    pub(crate) fn deserialise_payload_to_quorum_set(
        response: GetLatestMsgResponse,
    ) -> Option<QuorumSet> {
        let consensus_msg = if response.get_payload().is_empty() {
            None
        } else {
            let consensus_msg = match deserialize::<ConsensusMsg>(response.get_payload()) {
                Ok(cons_msg) => Some(cons_msg),
                Err(_) => None,
            };
            consensus_msg
        };
        if let Some(scp_msg) = consensus_msg {
            Some(scp_msg.scp_msg.quorum_set)
        } else {
            None
        }
    }

    /// 0. Add the reporting node to the set of crawled nodes
    /// 1. Add node to the set to discovered nodes
    /// 2. Iterate over all members of the Qset and add them to the set of peers that should be crawled
    pub(crate) fn handle_discovered_node(&mut self, crawled_node: String, node: &mut CrawledNode) {
        debug!("Handling crawled node {}..", crawled_node);
        self.to_crawl.remove(&crawled_node);
        self.crawled.insert(crawled_node);
        self.mobcoin_nodes.insert(node.clone());
        for member in node.quorum_set.nodes() {
            let address = format!("{}{}", "mc://", member.responder_id);
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
            let mut node_now_with_pk = node.clone();
            // Add the node to set already otherwise it will be left out of the report if 1. the
            // crawler does not know other nodes 2. it wasn't found in the other qsets 3. sth else
            // I haven't thought of
            let responder_id = format!("{}{}:{}", "mc://", node.domain, node.port);
            for other_node in self.mobcoin_nodes.iter() {
                if other_node != node {
                    for member in other_node.quorum_set.nodes() {
                        let address = format!("{}{}", "mc://", member.responder_id);
                        if node.public_key == Ed25519Public::default() && responder_id == address {
                            node_now_with_pk.public_key = member.public_key;
                            break;
                        }
                    }
                }
            }
            mobcoin_nodes_with_pks.insert(node_now_with_pk);
        }
        mobcoin_nodes_with_pks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_peer_address_to_cons_peer() {
        let peer = "localhost:443";
        let actual = Crawler::prepare_rpc(String::from(peer));
        assert!(actual.is_none());
    }

    #[test]
    fn correct_peer_address_to_cons_peer() {
        let peer = "mc://localhost:443";
        let actual = Crawler::prepare_rpc(String::from(peer));
        assert!(actual.is_some());
    }

    #[test]
    fn empty_msg_to_quorum_set() {
        let msg = GetLatestMsgResponse::new();
        let actual = Crawler::deserialise_payload_to_quorum_set(msg);
        assert!(actual.is_none());
    }

    #[test]
    fn record_new_node() {
        let mut crawler = Crawler::default();
        let reachable = false;
        let crawled_node_uri = String::from("mc://test.node:11");
        let mut crawled_node =
            CrawledNode::new(crawled_node_uri.clone(), reachable, QuorumSet::empty());
        crawler.handle_discovered_node(crawled_node_uri.clone(), &mut crawled_node);
        assert!(crawler.mobcoin_nodes.contains(&crawled_node));
        assert!(crawler.crawled.contains(&crawled_node_uri));
    }
}
