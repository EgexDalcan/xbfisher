use std::thread;
use std::time::Duration;

use crate::database::{db_update_last_alive, db_update_stations_list, get_stations, push_data_to_db, update_realtime_data};
use crate::filecontrol::read_config;
use crate::parsing::{parse_config_file, ConfigData};
use crate::station::Station;

/// Gets the list of stations from the database and puts them in a Vec to start data acquisition.
pub fn start_diag_data_from_db(config_data: &ConfigData) {
    let port = config_data.get_port();
    let interval = Duration::from_secs(config_data.get_diag_interval());
    loop {
        let mut svec: Vec<Station> = vec![];
        match get_stations(config_data.get_db_loc()) {
            Ok(lines) => {
                // Consumes the iterator, returns a String
                for line in lines.iter() {
                    if !line.is_empty() {
                        let linecut: Vec<&str> = line.split(" -").collect();
                        match Station::connect_station_by_ip(linecut[0].parse().expect("The database takes Integer for this column"), &linecut[1].into(), &linecut[2].into()) {
                            Ok(station) => svec.push(station),
                            Err(error) => eprintln!("Error while connecting to station: {error}"),
                        }
                    };
                }
                for i in &mut svec {
                    let data = i.gather_diag_data_set(port);
                    match push_data_to_db(data, config_data.get_db_loc()) {
                        Ok(_) => (),
                        Err(error) => eprintln!("Error while inserting data to the database: {error}"),
                    };
                    match update_realtime_data(data, config_data.get_db_loc()) {
                        Ok(_) => (),
                        Err(error) => eprintln!("Error while inserting data to the database: {error}"),
                    };
                }
            }
            Err( error) => eprintln!("Error while starting diagnostics data acquisition: {error}")
        }
        thread::sleep(interval);
    }
}

/// Gets a list of the stations from the database and puts them in a Vec to update their last_alive columns in the database.
pub fn update_last_alives(config_data: &ConfigData) {
    let port = config_data.get_port();
    let interval = Duration::from_secs(config_data.get_alive_interval());
    loop {
        let mut svec: Vec<Station> = Vec::new();
        match get_stations(config_data.get_db_loc()) {
            Ok(lines) => {
                // Consumes the iterator, returns a String
                for line in lines.iter() {
                    if !line.is_empty() {
                        let linecut: Vec<&str> = line.split(" -").collect();
                        match Station::connect_station_by_ip(linecut[0].parse().expect("The database takes Integer for this column"), &linecut[1].into(), &linecut[2].into()) {
                            Ok(station) => svec.push(station),
                            Err(error) => eprintln!("Error while connecting to station: {error}"),
                        }
                    };
                }
                for i in &mut svec {
                    match i.update_station_last_alive(port) {
                        Ok(station) => { db_update_last_alive(station, config_data.get_db_loc()).unwrap_or_else(|error| { eprintln!("Error while updating the last_alive column of station number: {}. Error: {error}", station.get_station_no()) }); },
                        Err(_) => (),
                    }
                }
            }
            Err( error) => eprintln!("Error while updating station last alive date: {error}")
        }
        thread::sleep(interval);
    }
}

pub fn update_program(interval: Duration) {
    loop {
        let config_data = parse_config_file(read_config());

        db_update_stations_list(&config_data);
        thread::sleep(interval);
    }
}