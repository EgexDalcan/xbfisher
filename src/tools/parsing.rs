use std::{fs::File, io};

use regex::Regex;

use crate::{station::{Station, StationData}, Error};

pub struct ConfigData {
    database_location: String,
    diag_interval: u64,
    alive_interval: u64,
    port: String,
    station_vec: Vec<Station>
}

impl ConfigData {
    pub fn get_svec(&self) -> &Vec<Station> {
        &self.station_vec
    }

    pub fn get_diag_interval(&self) -> u64 {
        self.diag_interval
    }

    pub fn get_alive_interval(&self) -> u64 {
        self.alive_interval
    }

    pub fn get_port(&self) -> &str {
        &self.port
    }

    pub fn get_db_loc(&self) -> &str {
        &self.database_location
    }
}

pub fn parse_diag_data(data: &Vec<String>) -> Result<StationData, Error> {
    if data.len() != 10 {
        eprintln!("Length of data is not 10. Length of data: {}", data.len());
        return Err(Error::ParsingError);
    }
    let mut data_list: Vec<String> = Vec::new();
    for line in data {
        let data_line = line.split_once("\n");
        if data_line.is_none() {
            return Err(Error::ParsingError);
        }
        let (_, rhs) = data_line.unwrap();
        data_list.push(rhs.to_string());
    }
    Ok(StationData::new(&data_list))
}

pub fn parse_config_file(file: io::Result<io::Lines<io::BufReader<File>>>) -> ConfigData {
    let mut svec: Vec<Station> = vec![];
    let mut diag_interval: u64 = 60;
    let mut alive_interval: u64 = 10;
    let mut port: String = "2537".to_string();
    let mut db_loc: String = "/etc/xbfisher/station_database.db3".to_string();

    if let Ok(lines) = file {
        // Consumes the iterator, returns a String
        for line in lines.flatten() {
            let com = Regex::new(r"^[#]").unwrap();
            if !line.is_empty() && !com.is_match(&line) {
                if line.contains("diag_data_interval=") {
                    diag_interval = match line.split("diag_data_interval=").last() {
                        Some(intv) => intv.trim().parse().unwrap_or_else(| error | {panic!("Invalid diagnostic data interval input. Make sure it is an unsigned integer. {error}")}),
                        None => 60
                    }
                } else if line.contains("station=") {
                    match line.split("station=").last() {
                        Some(stat) => {
                            let linecut: Vec<&str> = stat.split(" -").collect();
                            match Station::connect_station_by_ip(linecut[0].trim().parse().expect("The database takes Integer for this column."), &linecut[1].into(), &linecut[2].into()) {
                                Ok(station) => svec.push(station),
                                Err(error) => eprintln!("Error while connecting to station: {error}"),
                            }
                        }
                        None => panic!("Misconfigured station in config file. \"{}\" is not correctly configured. Use \"station=<no> -<name> -<ip_address>\" to set.", line),
                    }
                } else if line.contains("server_port=") {
                    match line.split("server_port=").last() {
                        Some(prt) => port = prt.to_string(),
                        None => ()
                    }
                } else if line.contains("check_alive_interval=") {
                    alive_interval = match line.split("check_alive_interval=").last() {
                        Some(intv) => intv.trim().parse().unwrap_or_else(| error | {panic!("Invalid life check interval input. Make sure it is an unsigned integer. {error}")}),
                        None => 10
                    }
                } else if line.contains("database_location=") {
                    db_loc = match line.split("database_location=").last() {
                        Some(loc) => loc.trim().to_string(),
                        None => {eprintln!("Invalid database location"); db_loc},
                    }
                } else {
                    panic!("Misconfigured config file. The line: {} is not correctly configured.", line)
                }
            };
        }
    } else {
        panic!{"Misconfigured config file."};
    }
    ConfigData { station_vec: svec, diag_interval: diag_interval, port: port, alive_interval: alive_interval, database_location: db_loc }
}