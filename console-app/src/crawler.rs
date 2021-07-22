use crate::core_types::*;
use crate::io::CrawlReport;

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
        CrawlReport::create_crawl_report(self)
    }

    /// Sends the given peer a gRPC and serialises its response into a QuorumSet.
    fn crawl_node(&mut self, peer: String) {
        info!("Crawling peer: {}", peer.clone());
        let mut reachable = false;
        let quorum_set = QuorumSet::empty();
        // We didn't even send the RPC so no need to take note of the node
        let rpc_client = match Self::prepare_rpc(peer.clone()) {
            None => {
                warn!("Terminating crawl on peer {} .", peer.clone());
                return;
            }
            Some(client) => client,
        };
        // RPC failure, e.g. no response
        // TODO: handle_discovered_node
        let rpc_response = match Self::send_rpc(rpc_client) {
            None => {
                warn!("Error in RPC response from {} .", peer.clone());
                let discovered = CrawledNode::new(peer.clone(), reachable, quorum_set);
                self.handle_discovered_node(peer.to_string(), discovered);
                return;
            }
            Some(reply) => {
                reachable = true;
                reply
            }
        };
        let quorum_set = match Self::deserialise_payload_to_quorum_set(rpc_response) {
            None => {
                warn!("Couldn't deserialise message from {}.", peer.clone());
                QuorumSet::empty()
            }
            Some(qs) => qs,
        };
        let discovered = CrawledNode::new(peer.clone(), reachable, quorum_set);
        self.handle_discovered_node(peer.to_string(), discovered);
    }

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
        let quorum_set = if let Some(msg) = consensus_msg {
            Some(msg.scp_msg.quorum_set)
        } else {
            None
        };
        quorum_set
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
