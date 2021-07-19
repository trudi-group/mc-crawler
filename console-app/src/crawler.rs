use crate::core_types::*;

use log::{debug, info, warn};
use std::time::Instant;
use std::{
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
};

use grpcio::{ChannelBuilder, EnvBuilder, Environment};
use mc_common::logger;
use mc_consensus_api::consensus_peer::GetLatestMsgResponse;
use mc_consensus_api::consensus_peer_grpc::ConsensusPeerApiClient;
use mc_consensus_api::empty;
use mc_consensus_scp::QuorumSet;
use mc_peers::{ConsensusMsg, Result as McResult};
use mc_util_grpc::ConnectionUriGrpcioChannel;
use mc_util_serial::{deserialize, serialize};
use mc_util_uri::ConsensusClientUri as ClientUri;

impl Crawler {
    pub fn crawl_network(&mut self) {
        let start = Instant::now();
        let crawler = Arc::new(Mutex::new(self));
        let mut handles = vec![];
        info!("Starting crawl..");
        while !crawler.lock().unwrap().to_crawl.is_empty() {
            for peer in crawler.lock().unwrap().to_crawl.drain() {
                info!("Crawling peer: {}", peer);
                let mut clone = Arc::clone(&crawler);
                let handle = thread::spawn(move || {
                    let rpc_client = match Self::prepare_rpc(peer) {
                        None => return,
                        Some(client) => client,
                    };
                    Self::send_rpc(rpc_client);
                });
                handles.push(handle);
            }
        }
        for threads in handles {
            threads.join().unwrap();
        }
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
                warn!("Error in RPC response from");
                None
            }
        };
        response
    }
}
