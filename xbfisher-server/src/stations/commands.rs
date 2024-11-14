use std::time::Duration;

use regex::Regex;

use crate::station::Station;
use crate::station;
use crate::tools::filecontrol;

pub fn parse_config(args: &[String]) -> (&str, &str, &str){
    let command = &args[1];
    let parameter1 = &args[2];
    let parameter2 = &args[3];
    (command, parameter1, parameter2)
}

pub fn start_data_from_no(stat_no: u8){
    let station = Station::connect_station(stat_no);
    let datavec = vec![station.gather_data_set()];
    filecontrol::write_data(datavec);
}

pub fn start_data_from_ip(ipaddr: &String, interval: &String){
    let station = Station::connect_station_by_ip(99, ipaddr);
    loop{
        let datavec = vec![station.gather_data_set()];
        filecontrol::write_data(datavec);
        std::thread::sleep(Duration::from_secs(interval.parse().unwrap()));
    }
}

/// Parses the "./hosts.txt" file and writes the gathered data from the stations named in "./hosts.txt" into a .csv file named after the date.
/// If no "./hosts.txt" exists, creates the file and panics.
/// interval: u64: designates the interval between different data retrievals in seconds.
pub fn start_data_from_list(interval: &String){
    let mut svec: Vec<Station> = vec![];
    if let Ok(lines) = filecontrol::read_lines("./hosts.txt".into()) {
        // Consumes the iterator, returns a String
        for line in lines.flatten() {
            let com = Regex::new(r"^[#]").unwrap();
            if !com.is_match(&line) & !line.is_empty() {
                let linecut: Vec<&str> = line.split(" -").collect();
                svec.push(Station::connect_station_by_ip(linecut[0].parse().unwrap(), &linecut[1].into()));
            };
        }
        loop {
            let mut datavec: Vec<station::DataRow> = vec![];
            for i in &svec{
                datavec.push(i.gather_data_set());
            }
            filecontrol::write_data(datavec);
            std::thread::sleep(Duration::from_secs(interval.parse().unwrap()));
        }
    }
}

pub fn get_current_data_from_no(stat_no: u8){
    let station = Station::connect_station(stat_no);
    let data_row = station.gather_data_set();
    println!("{}", data_row);
}

pub fn get_current_data_from_ip(ipaddr: &String){
    let station = Station::connect_station_by_ip(99, ipaddr);
    let data_row = station.gather_data_set();
    println!("{}", data_row);
}

pub fn get_station_network_interface(stat_no: u8){
    let station = Station::connect_station(stat_no);
    station.get_network_interfaces();
}

pub fn get_station_network_interface_from_ip(ipaddr: &String){
    let station = Station::connect_station_by_ip(99, ipaddr);
    station.get_network_interfaces();
}

pub fn ping_station(stat_no: u8, count: u16){
    let station = Station::connect_station(stat_no);
    station.ping_this_station(count);
}

pub fn ping_station_from_ip(ipaddr: &String, count: u16){
    let station = Station::connect_station_by_ip(99, ipaddr);
    station.ping_this_station(count);
}