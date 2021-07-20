use crate::core_types::*;

use log::{debug, info, warn};
use std::time::Instant;
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
};

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
    pub fn crawl_network(self) {
        let start = Instant::now();
        let crawler = Arc::new(Mutex::new(self));
        let mut handles = vec![];
        info!("Starting crawl..");
        loop {
            let set_lock = crawler.lock().expect("Mutex poisoned");
            let mut to_crawl_set = set_lock.to_crawl.write().expect("RWLock poisoned");
            for peer in to_crawl_set.drain() {
                info!("Crawling peer: {}", peer.clone());
                let clone = Arc::clone(&crawler);
                let mut reachable = false;
                let handle = thread::spawn(move || {
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
                            warn!("Terminating crawl on peer {} .", peer.clone());
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
                    let discovered = MobcoinNode::new(peer.clone(), reachable, quorum_set);
                    Self::handle_discovered_node(clone, peer.to_string(), discovered);
                });
                handles.push(handle);
            }
            let clone2 = Arc::clone(&crawler);
            if Self::should_quit(clone2) {
                break;
            }
        }
        for threads in handles {
            threads.join().unwrap();
        }
        let crawl_duration = start.elapsed();
        crawler.lock().unwrap().crawl_duration = crawl_duration;
        debug!("Crawler {:?}", crawler.lock().unwrap());
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

    fn should_quit(clone: Arc<Mutex<Crawler>>) -> bool {
        let crawler = clone.lock().expect("Mutex poisoned");
        let is_empty = crawler.to_crawl.read().unwrap().is_empty();
        is_empty
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
