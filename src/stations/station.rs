use std::time::Duration;
use rand::random;

use crate::parsing::parse_diag_data;
use crate::{math, req_comms, CommandKind, Error};
use crate::network::ping;

pub struct Station {
    pub station_no: u8,
    pub name: String,
    pub diag_data: StationData,
    pub ip_address: String,
}

impl Station{
    fn new_no(st_no: u8, st_name: &String, ipaddr: &String) -> Self {
        Self { station_no: st_no, name: st_name.to_string(), diag_data: StationData::new_empty(), ip_address: ipaddr.to_string()}
    }

    pub fn connect_station(stat_no: u8) -> Result<Self, Error> {
        let station = match stat_no{
            0 => Self::new_no(0, &"Frodo".to_string(), &"10.8.0.101".to_string()),
            1 => Self::new_no(1, &"Aragorn".to_string(), &"10.10.1.2".to_string()),
            2 => Self::new_no(2, &"Arwen".to_string(), &"10.10.2.2".to_string()),
            3 => Self::new_no(3, &"Gimli".to_string(), &"10.10.3.2".to_string()),
            4 => Self::new_no(4, &"Legolas".to_string(), &"10.10.4.2".to_string()),
            5 => Self::new_no(5, &"Bilbo".to_string(), &"10.10.5.2".to_string()),
            6 => Self::new_no(6, &"Galadriel".to_string(), &"10.10.6.2".to_string()),
            _ => panic!("An invalid station no!")
        };
        let timeout = Duration::from_secs(2);
        let connected = match ping::ping(
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
                true
            },
            Err(error) => {
                eprintln!("Problem during pinging Station {}. Station might be offline, or has a different address, otherwise you do not have connection. Error: {error}.", station.get_ip_address());
                false
            },
        };
        if connected {
            return Ok(station);
        }
        Err(Error::InvalidIPAdress)
    }

    pub fn connect_station_by_ip(st_no: u8, st_name: &String, ipaddr: &String) -> Result<Self, Error> {
        let station = Self::new_no(st_no, &st_name, ipaddr);
        let timeout = Duration::from_secs(2);
        let connected = match ping::ping(
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
                true
            },
            Err(error) => {
                eprintln!("Problem during pinging Station {st_no} with ip: {ipaddr}. Station might be offline, or has a different address, otherwise you do not have connection. Error: {error}.");
                false
            },
        };
        if connected {
            return Ok(station)
        }
        Err(Error::InvalidIPAdress)
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

    /// Gathers data from the station as StationData, pushes the data into the database and updates the struct.
    pub fn gather_diag_data_set(&mut self) -> &StationData{
        // Get the station data and assign its number, name, and latency.
        let latency = math::n_decimals(math::vec_mean(&self.ping_this_station_silent(5)), 4).to_string();

        let station_data = match req_comms(self, CommandKind::ReqDiag) {
            Ok(data) => parse_diag_data(&data),
            Err(err) => {eprintln!("Error while requesting diagnostics data: {err}"); Ok(StationData::new_error())},
        };

        let mut sdata = if let Err(error) = station_data {
            eprintln!("Error while parsing diagnostics data: {error}");
            StationData::new_error()
        } else {
            station_data.unwrap()
        };

        sdata.no = self.station_no.to_string();
        sdata.latency = latency;

        // Return StationData and update self.
        self.diag_data = sdata;
        &self.diag_data
    }
}

pub struct StationData {
    no: String,
    date: String,
    uptime: String,
    network_data: String,
    latency: String,
    socket_stats: String,
    memory: String,
    memory_details: String,
    swap: String,
    swap_details: String,
    cpu_load: String,
    load_avg: String,
    cpu_temp: String,
}

impl StationData {
    /// The &Vec<String> inputted here must be of length 9.
    pub fn new(data_list: &Vec<String>) -> StationData {
        if data_list.len() != 9 {
            Self::new_error();
        }
        let memory_details = data_list[4].clone().split("\nDetails: ").map(|x| x.to_string()).collect::<Vec<String>>();
        let swap_details = data_list[5].clone().split("\nDetails: ").map(|x| x.to_string()).collect::<Vec<String>>();
        Self {
            no: "-1".to_string(),
            date: data_list[0].clone(),
            uptime: data_list[1].clone(),
            network_data: data_list[2].clone(),
            latency: "-1".to_string(),
            socket_stats: data_list[3].clone(),
            memory: memory_details[0].clone(),
            memory_details: memory_details[1].clone(),
            swap: swap_details[0].clone(),
            swap_details: swap_details[1].clone(),
            cpu_load: data_list[6].clone(),
            load_avg: data_list[7].clone(),
            cpu_temp: data_list[8].clone(),
        }
    }

    pub fn new_error() -> StationData {
        return Self {
            no: "-1".to_string(),
            date: "Error".to_string(),
            uptime: "Error".to_string(),
            network_data: "Error".to_string(),
            latency: "Error".to_string(),
            socket_stats: "Error".to_string(),
            memory: "Error".to_string(),
            memory_details: "Error".to_string(),
            swap: "Error".to_string(),
            swap_details: "Error".to_string(),
            cpu_load: "Error".to_string(),
            load_avg: "Error".to_string(),
            cpu_temp: "Error".to_string(),
        }
    }

    pub fn new_empty() -> StationData {
        Self {
            no: "-1".to_string(),
            date: "Empty".to_string(),
            uptime: "Empty".to_string(),
            network_data: "Empty".to_string(),
            latency: "Empty".to_string(),
            socket_stats: "Empty".to_string(),
            memory: "Empty".to_string(),
            memory_details: "Empty".to_string(),
            swap: "Empty".to_string(),
            swap_details: "Empty".to_string(),
            cpu_load: "Empty".to_string(),
            load_avg: "Empty".to_string(),
            cpu_temp: "Empty".to_string(),
        }
    }

    pub fn output_data(&self) -> (String, String, String, String, String, String, String, String, String, String, String, String, String) {
        (self.no.clone(), self.date.clone(), self.uptime.clone(), self.network_data.clone(),
        self.latency.clone(), self.socket_stats.clone(), self.memory.clone(), self.memory_details.clone(),
        self.swap.clone(), self.swap_details.clone(), self.cpu_load.clone(), self.load_avg.clone(), self.cpu_temp.clone())
    }
}
