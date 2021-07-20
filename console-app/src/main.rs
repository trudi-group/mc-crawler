use env_logger::Env;

use mobcoin_crawler_console::*;

static BOOTSTRAP_PEER: &str = "mc://peer1.prod.mobilecoinww.com:443";

pub fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let crawler = core_types::Crawler::new(BOOTSTRAP_PEER);
    crawler.crawl_network();
}
