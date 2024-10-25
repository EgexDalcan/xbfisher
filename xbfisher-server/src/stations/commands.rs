use std::time::Duration;

use regex::Regex;

use crate::station::Station;
use crate::tools::filecontrol;

pub fn parse_config(args: &[String]) -> (&str, &str, &str){
    let command = &args[1];
    let parameter1 = &args[2];
    let parameter2 = &args[3];
    (command, parameter1, parameter2)
}

pub fn start_data_from_no(stat_no: u8, interval: u64){
    let station = Station::connect_station(stat_no);
    filecontrol::write_data(station, interval);
}

pub fn start_data_from_ip(ipaddr: String, interval: u64){
    let station = Station::connect_station_by_ip(99, ipaddr);
    filecontrol::write_data(station, interval);
}

pub fn start_data_from_list(interval: u64){
    let mut svec: Vec<Station> = vec![];
    if let Ok(lines) = filecontrol::read_lines("./hosts.txt".into()) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            let com = Regex::new(r"#").unwrap();
            if !com.is_match(&line) & !line.is_empty() {
                let linecut: Vec<&str> = line.split(" -").collect();
                svec.push(Station::connect_station_by_ip(linecut[0].parse().unwrap(), linecut[1].into()));
            };
        }
        loop {
            filecontrol::write_list_data(&svec, interval);
            std::thread::sleep(Duration::from_secs(interval));
        }
    }
}

pub fn get_current_data_from_no(stat_no: u8){
    let station = Station::connect_station(stat_no);
    let data_row = station.gather_data_set();
    println!("{}", data_row);
}

pub fn get_current_data_from_ip(ipaddr: String){
    let station = Station::connect_station_by_ip(99, ipaddr);
    let data_row = station.gather_data_set();
    println!("{}", data_row);
}

pub fn get_station_network_interface(stat_no: u8){
    let station = Station::connect_station(stat_no);
    station.get_network_interfaces();
}

pub fn get_station_network_interface_from_ip(ipaddr: String){
    let station = Station::connect_station_by_ip(99, ipaddr);
    station.get_network_interfaces();
}

pub fn ping_station(stat_no: u8, count: u16){
    let station = Station::connect_station(stat_no);
    station.ping_this_station(count);
}

pub fn ping_station_from_ip(ipaddr: String, count: u16){
    let station = Station::connect_station_by_ip(99, ipaddr);
    station.ping_this_station(count);
}