use std::time::Duration;

use regex::Regex;

use crate::database::{get_stations, push_data_to_db};
use crate::station::Station;
use crate::station;
use crate::tools::filecontrol;

pub fn parse_config(args: &[String]) -> (&str, &str, &str){
    let command = &args[1];
    let parameter1 = &args[2];
    let parameter2 = &args[3];
    (command, parameter1, parameter2)
}

/*pub fn start_data_from_no(stat_no: u8){
    let station = Station::connect_station(stat_no);
    let datavec = vec![station.gather_data_set()];
    filecontrol::write_data(datavec);
}*/

/*pub fn start_data_from_ip(usrname: &String, ipaddr: &String, interval: &String){
    let station = Station::connect_station_by_ip(99, usrname, ipaddr);
    loop{
        let datavec = vec![station.gather_data_set()];
        filecontrol::write_data(datavec);
        std::thread::sleep(Duration::from_secs(interval.parse().unwrap()));
    }
}*/

/// TODO: Used for testing for now, fix it to be used generally. Ignore interval parameter for now, we only take data once.
/// Gets the list of stations from the database and puts them in a Vec to start data acquisition.
/// interval: u64 = designates the interval between different data retrievals in seconds.
pub fn start_data_from_db_list(interval: &String) {
    let mut svec: Vec<Station> = vec![];
    match get_stations() {
        Ok(lines) => {
            // Consumes the iterator, returns a String
            println!("{:?}", lines);
            for line in lines.iter() {
                if !line.is_empty() {
                    let linecut: Vec<&str> = line.split(" -").collect();
                    svec.push(Station::connect_station_by_ip(linecut[0].parse().expect("The database takes Integer for this column."), &linecut[1].into(), &linecut[2].into()));
                };
            }
            for i in &mut svec {
                let _ = push_data_to_db(i.gather_diag_data_set());
            }
        }
        Err( error) => eprintln!("Error while starting diagnostics data acquisition: {error}")
        
    }
}

/*pub fn get_current_data_from_no(stat_no: u8){
    let mut station = Station::connect_station(stat_no);
    let data_row = station.gather_diag_data_set();
    println!("{}", data_row);
}

pub fn get_current_data_from_ip(usrname: &String, ipaddr: &String){
    let mut station = Station::connect_station_by_ip(99, usrname, ipaddr);
    let data_row = station.gather_diag_data_set();
    println!("{}", data_row);
}*/

pub fn ping_station(stat_no: u8, count: u16){
    let station = Station::connect_station(stat_no);
    station.ping_this_station(count);
}

pub fn ping_station_from_ip(st_name: &String, ipaddr: &String, count: u16){
    let station = Station::connect_station_by_ip(99, st_name, ipaddr);
    station.ping_this_station(count);
}