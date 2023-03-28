use env_logger::Env;
use log::{debug, info, warn};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use mc_crawler::{
    crawl,
    io::{CrawlReport, MobcoinFbas},
};

/// Crawl the MobileCoin Network and return the results in a JSON that can be passed to other programs
/// for further analysis.
#[derive(Debug, StructOpt)]
struct Opt {
    /// Path to directory where JSON(s) file should be saved.
    /// Defaults to "crawl_data/"
    #[structopt(short, long)]
    output: Option<PathBuf>,

    /// Set log level to debug, i.e. more log messages
    /// Default is info which contains less runtime messages
    /// Usage example "cargo run-- -d"
    #[structopt(short, long)]
    debug: bool,

    /// Output the found FBAS as JSON in stellarbeat.io format
    /// The JSON contains information about each node's hostname, port, QSet, public key and
    /// geolocation data when available.
    /// Usage example "cargo run-- --fbas"
    #[structopt(short, long)]
    fbas: bool,

    /// Provide complete crawl report as a JSON in addition to the FBAS written when only "--fbas" is
    /// passed.
    /// Usage example "cargo run-- -c"
    #[structopt(short, long)]
    complete: bool,

    /// Path to text file with the bootstrap nodes as URIs.
    /// Each node expected on a new line while lines beginning with "//" are understood to be
    /// comments and ignored.
    /// Will default to "./bootstrap.txt" if omitted.
    nodes_path: Option<PathBuf>,
}

fn create_output_dir(path: Option<&PathBuf>) -> Option<String> {
    let path_to_dir = if let Some(dir) = path {
        dir.as_path().display().to_string()
    } else {
        String::from("crawl_data")
    };
    if fs::create_dir_all(path_to_dir.clone()).is_ok() {
        Some(path_to_dir)
    } else {
        warn!("Error creating output directory..\nWill not create output files.");
        None
    }
}

fn write_report_to_file(output_dir: Option<String>, timestamp: String, report: CrawlReport) {
    if let Some(path_to_dir) = output_dir {
        let file_name = format!(
            "{}/{}{}{}",
            path_to_dir, "mobilecoin_crawl_report_", timestamp, ".json"
        );
        let file = File::create(file_name.clone()).expect("Error creating file");
        info!("Writing report to file {}", file_name);
        serde_json::to_writer_pretty(file, &report).expect("Error while writing report.");
    };
}

fn write_fbas_to_file(output_dir: Option<String>, timestamp: String, fbas: MobcoinFbas) {
    if let Some(path_to_dir) = output_dir {
        let file_name = format!(
            "{}/{}{}{}",
            path_to_dir, "mobilecoin_nodes_", timestamp, ".json"
        );
        let file = File::create(file_name.clone()).expect("Error creating file");
        info!("Writing fbas to file {}", file_name);
        serde_json::to_writer_pretty(file, &fbas).expect("Error while writing report.");
    };
}

fn read_bs_peers(path: Option<&PathBuf>) -> Vec<String> {
    let mut bs_peers: Vec<String> = vec![];
    let file_path = if let Some(bs_path) = path {
        bs_path.as_path()
    } else {
        Path::new("bootstrap.txt")
    };
    let file = File::open(file_path).expect("Error opening bootstrap file.");
    info!("Reading bootstrap peers from {:?}", file_path);
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let peer_uri = line.expect("Error while reading peer URI.");
        if peer_uri.starts_with("//") {
            continue;
        }
        bs_peers.push(peer_uri)
    }
    debug!("Read {} bootstrap peers.", bs_peers.len());
    bs_peers
}

pub fn main() {
    let args = Opt::from_args();
    let log_level = if args.debug { "debug" } else { "info" };
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", log_level)
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let bs_peers = read_bs_peers(args.nodes_path.as_ref());
    let mut crawler = crawl::Crawler::new(bs_peers);
    crawler.crawl_network();
    
	
    
    if args.fbas || args.complete {
        let output_dir = create_output_dir(args.output.as_ref());
        if output_dir.is_some() {
            let fbas = MobcoinFbas::create_mobcoin_fbas(&crawler);
            if args.fbas {
                write_fbas_to_file(output_dir.clone(), crawler.crawl_time.clone(), fbas.clone());
            }
            if args.complete {
                let report = CrawlReport::create_crawl_report(fbas, &crawler);
                write_report_to_file(output_dir, crawler.crawl_time, report);
            }
        }
    }
}
