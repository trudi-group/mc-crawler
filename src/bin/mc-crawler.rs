use env_logger::Env;
use log::info;
use std::fs;
use std::fs::File;

use mc_crawler::*;

static BOOTSTRAP_PEER: &str = "mc://peer1.prod.mobilecoinww.com:443";

pub fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let mut crawler = core_types::Crawler::new(BOOTSTRAP_PEER);
    let report = crawler.crawl_network();
    let dir = "output";
    fs::create_dir_all(dir).expect("Error creating output directory");
    let path = format!(
        "{}/{}{}{}",
        dir, "mobilecoin_nodes_", crawler.crawl_time, ".json"
    );
    let file = File::create(path.clone()).expect("Error creating file");
    info!("Writing report to file {}", path);
    serde_json::to_writer(file, &report).expect("Error while writing report.");
}
