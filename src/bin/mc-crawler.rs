use env_logger::Env;
use log::info;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

use mc_crawler::{crawl, io::CrawlReport};

static BOOTSTRAP_PEER: &str = "mc://peer1.prod.mobilecoinww.com:443";

/// Crawl the MobileCoin Network and return the results in a JSON (and an optional CSV) that can be passed to other programs
/// for further analysis.
#[derive(Debug, StructOpt)]
struct Opt {
    /// Path to directory where JSON file should be saved.
    /// Defaults to "crawl_data/"
    #[structopt(short, long)]
    output: Option<PathBuf>,

    /// Set log level to debug, i.e. more log messages
    /// Default is info which contains less runtime messages
    /// Usage example "cargo run-- -d"
    #[structopt(short, long)]
    debug: bool,

    /// Provide additional crawl report as a CSV file.
    /// It is written to the "output" directory in addition to the JSON.
    #[structopt(long)]
    csv: bool,
}

fn write_crawl_report_to_file(
    path: Option<&PathBuf>,
    write_csv: bool,
    timestamp: String,
    report: CrawlReport,
) {
    let path_to_dir = if let Some(dir) = path {
        dir.as_path().display().to_string()
    } else {
        String::from("crawl_data")
    };
    fs::create_dir_all(path_to_dir.clone()).expect("Error creating output directory");
    let json_file_name = format!(
        "{}/{}{}{}",
        path_to_dir, "mobilecoin_nodes_", timestamp, ".json"
    );
    let json_file = File::create(json_file_name.clone()).expect("Error creating JSON file");
    info!("Writing JSON report to file {}", json_file_name);
    report.write_json_report_to_file(json_file);
    if write_csv {
        let csv_file_name = format!(
            "{}/{}{}{}",
            path_to_dir, "mobilecoin_nodes_", timestamp, ".csv"
        );
        let csv_report = report.to_csv_report();
        info!("Writing CSV report to file {}", csv_file_name);
        csv_report.write_csv_report_to_file(csv_file_name);
    }
}

pub fn main() {
    let args = Opt::from_args();
    let log_level = if args.debug { "debug" } else { "info" };
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", log_level)
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let mut crawler = crawl::Crawler::new(BOOTSTRAP_PEER);
    let report = crawler.crawl_network();
    write_crawl_report_to_file(args.output.as_ref(), args.csv, crawler.crawl_time, report);
}
