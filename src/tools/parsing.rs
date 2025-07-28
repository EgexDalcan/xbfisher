use std::{fs::File, io};

use regex::Regex;

use crate::{station::{Station, StationData}, Error};

pub struct ConfigData {
    interval: u64,
    station_vec: Vec<Station>
}

impl ConfigData {
    pub fn get_svec(&self) -> &Vec<Station> {
        &self.station_vec
    }

    pub fn get_interval(&self) -> u64 {
        self.interval
    }
}

pub fn parse_diag_data(data: &Vec<String>) -> Result<StationData, Error> {
    if data.len() != 9 {
        eprintln!("Length of data is not 9. Length of data: {}", data.len());
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
    let mut interval: u64 = 60;
    if let Ok(lines) = file {
        // Consumes the iterator, returns a String
        for line in lines.flatten() {
            let com = Regex::new(r"^[#]").unwrap();
            if !line.is_empty() && !com.is_match(&line) {
                if line.contains("diag_data_interval=") {
                    interval = match line.split("diag_data_interval=").last() {
                        Some(intv) => intv.trim().parse().unwrap_or_else(| error | {panic!("Invalid interval input. Make sure it is an unsigned integer. {error}")}),
                        None => 60
                    }
                } else {
                    let linecut: Vec<&str> = line.split(" -").collect();
                    match Station::connect_station_by_ip(linecut[0].trim().parse().expect("The database takes Integer for this column."), &linecut[1].into(), &linecut[2].into()) {
                        Ok(station) => svec.push(station),
                        Err(error) => eprintln!("Error while connecting to station: {error}"),
                    }
                }
            };
        }
    } else {
        panic!{"Misconfigured config file."};
    }
    ConfigData { station_vec: svec, interval: interval }
}