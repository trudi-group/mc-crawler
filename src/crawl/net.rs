use crate::crawl::core_types::*;

use chrono::{DateTime, Utc};
use log::{info, warn};
use std::time::Instant;

use mc_consensus_api::{
    consensus_peer::GetLatestMsgResponse, consensus_peer_grpc::ConsensusPeerApiClient, empty,
};
use mc_consensus_scp::QuorumSet;

impl Crawler {
    /// This loop controls the entire crawl.
    /// The crawl ends when there are no more peers in the queue.
    /// We call get_public_keys_from_quorum_sets in order to get fill the MobcoinFbas with PK instead of hostnames.
    /// The MobcoinFbas contains all nodes that were found ready to be written as a JSON.
    pub fn crawl_network(&mut self) -> &mut Self {
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
        self
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
        let rpc_success = match Self::send_rpc(rpc_client) {
            None => {
                warn!("Failure sending RPC to {} .", peer);
                None
            }
            Some(reply) => {
                self.reachable_nodes += 1;
                reachable = true;
                Some(reply)
            }
        };
        let qset = match rpc_success {
            Some(rpc_response) => match Self::deserialise_payload_to_quorum_set(rpc_response) {
                None => {
                    warn!("Couldn't deserialise message from {}.", peer);
                    QuorumSet::empty()
                }
                Some(qs) => qs,
            },
            None => quorum_set,
        };
        let mut crawled = CrawledNode::new(peer.clone(), reachable, qset);
        self.handle_discovered_node(peer, &mut crawled);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_with_empty_queue_wont_panic() {
        let mut crawler = Crawler::default();
        crawler.crawl_network();
        assert!(crawler.mobcoin_nodes.is_empty());
        assert!(crawler.to_crawl.is_empty());
        assert_eq!(crawler.reachable_nodes, 0);
    }
}
