use core::{fmt, str};
use std::process::Command;
use std::time::Duration;
use rand::random;
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
}

impl fmt::Display for DataRow{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Time of Day: {}, Latency: {} ms, CPU Temp: {} C\n", self.time, self.ping_latency, self.cpu_temperature)
    }
}

pub struct Station{
    pub station_no: u8,
    pub ip_address: String,
    pub usr_name: String,
}

impl Station{
    fn new_no(st_no: u8, usr_name: &String, ipaddr: &String) -> Self{
        Self { station_no: st_no, ip_address: ipaddr.to_string(), usr_name: usr_name.to_string()}
    }

    pub fn connect_station(stat_no: u8) -> Self{
        let station = match stat_no{
            0 => Self::new_no(0, &"frodo_central".into(), &"10.8.0.101".to_string()),
            1 => Self::new_no(1, &"pi".into(), &"10.10.1.2".to_string()),
            2 => Self::new_no(2, &"pi".into(), &"10.10.2.2".to_string()),
            3 => Self::new_no(3, &"pi".into(), &"10.10.3.2".to_string()),
            4 => Self::new_no(4, &"pi".into(), &"10.10.4.2".to_string()),
            5 => Self::new_no(5, &"pi".into(), &"10.10.5.2".to_string()),
            6 => Self::new_no(6, &"pi".into(), &"10.10.6.2".to_string()),
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

    pub fn connect_station_by_ip(st_no: u8, username: &String, ipaddr: &String) -> Self{
        let station = Self::new_no(st_no, username, ipaddr);
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

    pub fn get_user_name(&self) -> &String {
        &self.usr_name
    }

    pub fn ping_this_station(&self, count: u16) -> Vec<f32>{
        ping::ping_station(self, count)
    }

    fn ping_this_station_silent(&self, count: u16) -> Vec<f32>{
        ping::ping_station_silent(self, count)
    }

    pub fn get_current_temperature(&self) -> Result<String, Error>{
        let mut remote_data = Command::new("ssh");
        let username_ip = format!("{}@{}", self.get_user_name(), self.get_ip_address());
        let home_dir = format!("/home/hea-data/.ssh/id_rsa");
        // We have a station with a broken temp detector so we are using secondary temp detector if its that station.
        let loc = match self.get_ip_address().as_str() {
            "10.8.0.110" => format!("/sys/class/thermal/thermal_zone1/temp"),
            _ => format!("/sys/class/thermal/thermal_zone0/temp")
        };
        let data = remote_data.args(["-i", home_dir.as_str(), username_ip.as_str(),"cat", loc.as_str()]).output().unwrap_or_else(|error|{panic!("Error: {error}")});
        let array = &*data.stdout;
        match str::from_utf8(array).unwrap().trim().parse::<i32>() {
            Ok(a) => Ok(math::n_decimals(a as f32 /1000.0, 4).to_string()),
            Err(_x) => Err(Error::InternalError) ,
        }
    }

    /// Gathers data from the station and returns it as DataRow
    pub fn gather_data_set(&self) -> DataRow{
        let date: chrono::DateTime<Local> = chrono::offset::Local::now();
        let latency = math::n_decimals(math::vec_mean(&self.ping_this_station_silent(5)), 4).to_string();
        DataRow{
            no: self.station_no.to_string(),
            ping_latency: latency,
            cpu_temperature: match self.get_current_temperature(){
                Ok(a) => a,
                Err(error) => format!("Error: {}", error).to_string()
            },
            time: format!("{}:{}", date.hour(), date.minute()),
        }
    }
}
