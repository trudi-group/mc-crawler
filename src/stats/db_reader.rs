use log::debug;
use maxminddb::{geoip2, MaxMindDBError};
use std::net::IpAddr;

static ISP_DB_PATH: &str = "./src/stats/geolite2_dbs/GeoLite2-ASN_20210727/GeoLite2-ASN.mmdb";

static COUNTRY_DB_PATH: &str =
    "./src/stats/geolite2_dbs/GeoLite2-Country_20210727/GeoLite2-Country.mmdb";

pub struct DbReader {
    reader: maxminddb::Reader<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum Database {
    Asn,
    Country,
}

impl DbReader {
    pub fn new(db: Database) -> Self {
        let path = match db {
            Database::Asn => ISP_DB_PATH,
            Database::Country => COUNTRY_DB_PATH,
        };
        let reader = maxminddb::Reader::open_readfile(path).expect("Error opening database");
        debug!("Succesfully opened {:?} database", db);
        DbReader { reader }
    }

    pub fn lookup_country(&self, ip: IpAddr) -> String {
        let country: Result<geoip2::Country, MaxMindDBError> = self.reader.lookup(ip);
        match country {
            Ok(country_data) => {
                let country_map = country_data.country.and_then(|cy| cy.names);
                if let Some(map) = country_map {
                    map.get("en").unwrap().to_string()
                } else {
                    warn!("EN entry unavailable.");
                    "".to_string()
                }
            }
            Err(err) => {
                warn!("Country lookup for {} failed: {}", ip, err);
                "".to_string()
            }
        }
    }

    pub fn lookup_isp(&self, ip: IpAddr) -> String {
        let isp: Result<geoip2::Isp, MaxMindDBError> = self.reader.lookup(ip);
        match isp {
            Ok(isp_info) => {
                if let Some(isp_name) = isp_info.isp {
                    isp_name.to_string()
                } else {
                    warn!("No ISP name entry found for {} in database.", ip);
                    "".to_string()
                }
            }
            Err(err) => {
                warn!("ISP lookup for {} failed: {}", ip, err);
                "".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn valid_ip_country_lookup() {
        let db_reader = DbReader::new(Database::Country);
        let google: IpAddr = FromStr::from_str("8.8.8.8").unwrap();
        let actual = db_reader.lookup_country(google);
        let expected = String::from("United States");
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_ip_country_lookup() {
        let db_reader = DbReader::new(Database::Country);
        let zero_addr: IpAddr = FromStr::from_str("0.0.0.0").unwrap();
        let actual = db_reader.lookup_country(zero_addr);
        let expected = String::from("");
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_ip_isp_lookup() {
        let db_reader = DbReader::new(Database::Asn);
        let zero_addr: IpAddr = FromStr::from_str("0.0.0.0").unwrap();
        let actual = db_reader.lookup_isp(zero_addr);
        let expected = String::from("");
        assert_eq!(actual, expected);
    }
}
