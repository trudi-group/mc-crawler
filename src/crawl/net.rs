use crate::crawl::core_types::*;

use chrono::{DateTime, Utc};
use log::{info, warn};
use std::time::Instant;

use mc_consensus_api::{
    consensus_common::LastBlockInfoResponse, consensus_common_grpc::BlockchainApiClient,
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
                self.crawl_node(peer);
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
    fn crawl_node(&mut self, peer: &String) {
        info!("Crawling peer: {}", peer);

        if let (Some(consensus_client), Some(blockchain_client)) = Self::prepare_rpc(peer.clone()) {
            let mut reachable = false;
            let quorum_set =
                if let Some(rpc_reply) = Self::send_rpc_get_latest_msg(consensus_client) {
                    self.reachable_nodes += 1;
                    reachable = true;
                    if let Some(qs) = Self::deserialise_payload_to_quorum_set(rpc_reply) {
                        qs
                    } else {
                        warn!("Couldn't deserialise message from {}.", peer);
                        QuorumSet::empty()
                    }
                } else {
                    warn!("Failure sending RPC to {} .", peer);
                    QuorumSet::empty()
                };

            let (network_block_version, latest_block) =
                if let Some(rpc_reply) = Self::send_rpc_get_last_block_info(blockchain_client) {
                    let network_block_version = rpc_reply.get_network_block_version();
                    let index = rpc_reply.get_index();
                    (network_block_version, index)
                } else {
                    warn!(
                        "Couldn't get network block version and latest block from {}.",
                        peer
                    );
                    (0, 0)
                };
            let mut crawled = CrawledNode::new(
                peer.clone(),
                reachable,
                quorum_set,
                latest_block,
                network_block_version,
            );
            self.handle_discovered_node(peer, &mut crawled);
        } else {
            // We didn't even send the RPC so no need to take note of the node
            warn!("Terminating crawl on peer {} .", peer);
        };
    }

    /// The RPC "get_latest_msg" expects an empty protobuf and returns the last ConsensusMsg a node
    /// sent (see
    /// https://github.com/mobilecoinfoundation/mobilecoin/blob/master/peers/src/consensus_msg.rs#L20 for the exact definition)
    fn send_rpc_get_latest_msg(client: ConsensusPeerApiClient) -> Option<GetLatestMsgResponse> {
        let response = match client.get_latest_msg(&empty::Empty::default()) {
            Ok(reply) => Some(reply),
            Err(_) => {
                warn!("Error in RPC response.");
                None
            }
        };
        response
    }
    fn send_rpc_get_last_block_info(client: BlockchainApiClient) -> Option<LastBlockInfoResponse> {
        let response = match client.get_last_block_info(&empty::Empty::default()) {
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
        assert_eq!(crawler.crawled.len(), 0);
    }
}
