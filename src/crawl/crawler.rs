use crate::crawl::core_types::*;
use crate::io::report::CrawlReport;

use chrono::{DateTime, Utc};
use log::{info, warn};
use std::time::Instant;
use std::{str::FromStr, sync::Arc};

use grpcio::{ChannelBuilder, EnvBuilder};
use mc_common::logger;
use mc_consensus_api::consensus_peer::GetLatestMsgResponse;
use mc_consensus_api::consensus_peer_grpc::ConsensusPeerApiClient;
use mc_consensus_api::empty;
use mc_consensus_scp::QuorumSet;
use mc_peers::ConsensusMsg;
use mc_util_grpc::ConnectionUriGrpcioChannel;
use mc_util_serial::deserialize;
use mc_util_uri::ConsensusClientUri as ClientUri;

impl Crawler {
    /// This loop controls the entire crawl.
    /// The crawl ends when there are no more peers in the queue.
    /// We call get_public_keys_from_quorum_sets in order to get fill the CrawlReport with PK instead of hostnames.
    /// The CrawlReport contains all nodes that were found ready to be written as a JSON.
    pub fn crawl_network(&mut self) -> CrawlReport {
        let start = Instant::now();
        let now: DateTime<Utc> = Utc::now();
        info!("Starting crawl..");
        loop {
            for peer in self.to_crawl.clone().iter() {
                self.crawl_node(peer.to_string());
            }
            if self.to_crawl.is_empty() {
                break;
            }
        }
        self.crawl_duration = start.elapsed();
        self.crawl_time = now.to_rfc3339();
        info!(
            "Crawl Summary - Crawled nodes: {}, Crawl Duration {:?}",
            self.crawled.len(),
            self.crawl_duration
        );
        let nodes_with_pks = self.get_public_keys_from_quorum_sets();
        self.mobcoin_nodes = nodes_with_pks;
        CrawlReport::create_crawl_report(self)
    }

    /// 1. Sends the given peer a gRPC.
    /// 2. Get its QSet.
    /// 3. Call the handle_discovered_node method on the peer.
    fn crawl_node(&mut self, peer: String) {
        info!("Crawling peer: {}", peer);
        let mut reachable = false;
        let quorum_set = QuorumSet::empty();
        // We didn't even send the RPC so no need to take note of the node
        let rpc_client = match Self::prepare_rpc(peer.clone()) {
            None => {
                warn!("Terminating crawl on peer {} .", peer);
                return;
            }
            Some(client) => client,
        };
        let rpc_response = match Self::send_rpc(rpc_client) {
            None => {
                warn!("Error in RPC response from {} .", peer);
                let mut discovered = CrawledNode::new(peer.clone(), reachable, quorum_set);
                self.handle_discovered_node(peer, &mut discovered);
                return;
            }
            Some(reply) => {
                reachable = true;
                reply
            }
        };
        let quorum_set = match Self::deserialise_payload_to_quorum_set(rpc_response) {
            None => {
                warn!("Couldn't deserialise message from {}.", peer);
                QuorumSet::empty()
            }
            Some(qs) => qs,
        };
        let mut discovered = CrawledNode::new(peer.clone(), reachable, quorum_set);
        self.handle_discovered_node(peer, &mut discovered);
    }

    /// Opens an RPC channel to the peer which can be used for communication later
    fn prepare_rpc(peer: String) -> Option<ConsensusPeerApiClient> {
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

    /// The RPC "get_latest_msg" expects an empty protobuf and returns the last ConsensusMsg a node
    /// sent (see
    /// https://github.com/mobilecoinfoundation/mobilecoin/blob/master/peers/src/consensus_msg.rs#L20 for the exact definition)
    fn send_rpc(client: ConsensusPeerApiClient) -> Option<GetLatestMsgResponse> {
        let response = match client.get_latest_msg(&empty::Empty::default()) {
            Ok(reply) => Some(reply),
            Err(_) => {
                warn!("Error in RPC response.");
                None
            }
        };
        response
    }

    /// The bytes of the RPC response is deserialised into an McQuorumSet::QuorumSet
    fn deserialise_payload_to_quorum_set(payload: GetLatestMsgResponse) -> Option<QuorumSet> {
        let consensus_msg = if payload.get_payload().is_empty() {
            None
        } else {
            let msg = match deserialize::<ConsensusMsg>(payload.get_payload()) {
                Ok(cons) => Some(cons),
                Err(_) => None,
            };
            msg
        };
        if let Some(msg) = consensus_msg {
            Some(msg.scp_msg.quorum_set)
        } else {
            None
        }
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
}
