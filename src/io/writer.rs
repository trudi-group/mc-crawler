use crate::io::{CrawlReport, CsvReport};
use csv::Writer;
use std::fs::File;

impl CrawlReport {
    /// Writes an io::CrawlReport to the JSON file specified json_file_path
    pub fn write_json_report_to_file(&self, json_file_path: File) {
        serde_json::to_writer_pretty(json_file_path, &self).expect("Error while writing JSON.");
    }
}
impl CsvReport {
    /// Writes an io::CsvReport to the CSV file specified csv_file_path
    pub fn write_csv_report_to_file(&self, csv_file_path: String) {
        let mut writer = Writer::from_path(csv_file_path).expect("Error while creating CSV file.");
        for node in self.0.iter() {
            writer.serialize(node).expect("Error while writing CSV.");
            writer.flush().expect("Error while writing CSV.");
        }
    }
}
