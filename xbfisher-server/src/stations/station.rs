use core::fmt;
use std::{thread, time::Duration};
use rand::random;
use systemstat::{System, Platform};
use chrono::{Local, Timelike};

use crate::{math, Error};
use crate::pinging::ping;

#[derive(serde::Serialize)]
pub struct DataRow{
    #[serde(rename = "Time")]
    time: String,
    #[serde(rename = "Station No")]
    no: String,
    #[serde(rename = "Latency")]
    ping_latency: String,
    #[serde(rename = "CPU Temperature")]
    cpu_temperature: String,
    #[serde(rename = "CPU Load")]
    cpu_load: String,
}

impl fmt::Display for DataRow{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Time of Day: {}, Latency: {} ms, CPU Temp: {} C, CPU Load: {}\n", self.time, self.ping_latency, self.cpu_temperature, self.cpu_load)
    }
}

pub struct Station{
    pub station_no: u8,
    pub ip_address: String,
}

impl Station{
    fn new_no(st_no: u8, ipaddr: &String) -> Self{
        Self { station_no: st_no, ip_address: ipaddr.to_string() }
    }

    pub fn connect_station(stat_no: u8) -> Self{
        let station = match stat_no{
            0 => Self::new_no(0, &"10.8.0.101".to_string()),
            1 => Self::new_no(1, &"10.10.1.2".to_string()),
            2 => Self::new_no(2, &"10.10.2.2".to_string()),
            3 => Self::new_no(3, &"10.10.3.2".to_string()),
            4 => Self::new_no(4, &"10.10.4.2".to_string()),
            5 => Self::new_no(5, &"10.10.5.2".to_string()),
            6 => Self::new_no(6, &"10.10.6.2".to_string()),
            _ => panic!("An invalid station no!")
        };
        let timeout = Duration::from_secs(2);
        match ping::ping(
            station.get_ip_address().parse().unwrap_or_else(|error|{
                panic!("Error reading this address: \"{}\". check if its correct. Error: {error}", station.get_ip_address());
            }),
            Some(timeout),
            Some(166),
            Some(3),
            Some(5),
            Some(&random()),
        ){
            Ok(_a) => {
                println!("Station found, initiating connection.");
            },
            Err(error) => {
                println!("Problem during pinging Station {}. Station might be offline, or has a different address, otherwise you do not have connection. Error: {error}.", station.get_ip_address());
            },
        };
        station
    }

    pub fn connect_station_by_ip(st_no: u8, ipaddr: String) -> Self{
        let station = Self::new_no(st_no, &ipaddr);
        let timeout = Duration::from_secs(2);
        match ping::ping(
            ipaddr.parse().unwrap_or_else(|error|{
                panic!("Error reading this address: \"{ipaddr}\". check if its correct. Error: {error}");
            }),
            Some(timeout),
            Some(166),
            Some(3),
            Some(5),
            Some(&random()),
        ){
            Ok(_a) => {
                println!("Station {st_no} with ip: {ipaddr} found. Initiating connection.");
            },
            Err(error) => {
                println!("Problem during pinging Station {st_no} with ip: {ipaddr}. Station might be offline, or has a different address, otherwise you do not have connection. Error: {error}.");
            },
        };
        station
    }

    pub fn get_ip_address(&self) -> &String {
        &self.ip_address
    }

    pub fn get_station_no(&self) -> u8 {
        self.station_no
    }

    pub fn ping_this_station(&self, count: u16) -> Vec<f32>{
        ping::ping_station(self, count)
    }

    fn ping_this_station_silent(&self, count: u16) -> Vec<f32>{
        ping::ping_station_silent(self, count)
    }

    pub fn get_current_temperature(&self) -> Result<f32, Error> {
        let sys = System::new();
        match sys.cpu_temp() {
            Ok(cpu_temp) => Ok(cpu_temp),
            Err(x) => Err(Error::IoError { error: (x) }),
        }
    }

    pub fn get_current_cpu_load(&self) -> Result<f32, Error>{
        let sys = System::new();
        match sys.cpu_load_aggregate() {
            Ok(cpu) => {
                thread::sleep(Duration::from_secs(1));
                let cpu = cpu.done().unwrap();
                Ok(100.0 - (cpu.idle * 100.0))
            },
            Err(x) => Err(Error::IoError { error: (x) })
        }
    }

    pub fn get_network_interfaces(&self){
        let sys = System::new();
        match sys.networks() {
            Ok(netifs) => {
                let mut s1: String = String::new();
                for netif in netifs.values() {
                    let s2 = format!("{} ({:?})\n", netif.name, netif.addrs);
                    s1 = s1 + &s2;
                }
                println!("{}", s1);
            }
            Err(x) => panic!("Error getting the network interfaces! Error: {}", x)
        }
    }

    /// Gathers data from the station and returns it as DataRow
    pub fn gather_data_set(&self) -> DataRow{
        let date: chrono::DateTime<Local> = chrono::offset::Local::now();
        let latency = ping::vec_mean(&self.ping_this_station_silent(5)).to_string();
        DataRow{
            no: self.station_no.to_string(),
            ping_latency: latency,
            cpu_temperature: match self.get_current_temperature(){
                Ok(a) => math::n_decimals(a, 4).to_string(),
                Err(error) => format!("Error: {}", error).to_string()
            },
            cpu_load: match self.get_current_cpu_load(){
                Ok(a) => format!("{}%", math::n_decimals(a, 4)),
                Err(error) => format!("Error: {}", error).to_string()
            },
            time: format!("{}:{}", date.hour(), date.minute()),
        }
    }
}

