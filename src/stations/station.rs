use core::f32;
use std::time::Duration;
use chrono::{DateTime, Local, Utc};
use rand::random;
use rusqlite::types::Value;

use crate::network::tcpclient::ReturnKind;
use crate::parsing::DiagData;
use crate::{math, req_comms, CommandKind, Error};
use crate::network::ping;

pub struct Station {
    pub station_no: u8,
    pub name: String,
    pub diag_data: StationData,
    pub ip_address: String,
    pub last_alive: DateTime<Local>,
}

impl Station{
    pub fn new(st_no: u8, st_name: &String, ipaddr: &String) -> Self {
        Self { station_no: st_no, name: st_name.to_string(), diag_data: StationData::new_empty(), ip_address: ipaddr.to_string(), last_alive: Local::now()}
    }

    pub fn connect_station(st_no: u8, st_name: &String, ipaddr: &String) -> Result<Self, Error> {
        let station = Self::new(st_no, &st_name, ipaddr);
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
    pub fn gather_diag_data_set(&mut self, port: &str) -> &StationData{
        // Get the station data and assign its number, name, and latency.
        let latency = math::n_decimals(math::vec_mean(&self.ping_this_station_silent(5)), 4);

        let mut station_data = match req_comms(&self, CommandKind::ReqDiag, port) {
            ReturnKind::DiagRet(data) => StationData::new(data),
            ReturnKind::Err(err) => { eprintln!("Error while requesting diagnostics data: {err}"); StationData::new_error() },
            ReturnKind::AliveRet => { eprintln!("Received ReturnKind::AliveRet to the DiagData request. Should be impossible.") ; StationData::new_error() }
        };

        station_data.no = self.station_no as i32;
        station_data.latency = latency;

        // Return StationData and update self.
        self.diag_data = station_data;
        &self.diag_data
    }

    pub fn update_station_last_alive(&mut self, port: &str) -> Result<&Station, Error> {
        match req_comms(&self, CommandKind::CheckAlive, port) {
            ReturnKind::AliveRet => { self.last_alive = Local::now(); return Ok(self) },
            ReturnKind::Err(error) => { eprintln!("Error while requesting life status from station {}. Error: {error}", self.station_no); return Err(Error::InvalidTCPCommunication)},
            ReturnKind::DiagRet(_) => { eprintln!("Received ReturnKind::DiagRet to the CheckAlive request. Should be impossible.") ; return Err(Error::InvalidTCPCommunication)}
        }
    }
}

pub struct StationData {
    no: i32,
    date: Duration,
    uptime: i32,
    interface_data: String,
    latency: f32,
    socket_stats: String,
    memory_used: i64,
    memory_max: i64,
    memory_details: String,
    swap_used: i64,
    swap_max: i64,
    swap_details: String,
    cpu_load_user: f32,
    cpu_load_system: f32,
    cpu_load_idle: f32,
    load_onem_avg: f32,
    load_fivem_avg: f32,
    load_fifteenm_avg: f32,
    cpu_temp: f32,
    disk_use: String,
}

impl StationData {
    pub fn new(data_list: DiagData) -> StationData {
        let mut disk_data = String::new();
        for disk in data_list.disks {
            disk_data = disk_data + &disk.get_data() + "\n";
        }

        Self {
            no: 0,
            date: Duration::from_nanos(data_list.date as u64),
            uptime: data_list.uptime as i32,
            interface_data: data_list.networks,
            latency: 0.0,
            socket_stats: data_list.socket_stats,
            memory_used: data_list.memory_used as i64,
            memory_max: data_list.memory_max as i64,
            memory_details: data_list.memory_details,
            swap_used: data_list.swap_used as i64,
            swap_max: data_list.swap_max as i64,
            swap_details: data_list.swap_details,
            cpu_load_user: data_list.cpu_user,
            cpu_load_system: data_list.cpu_system,
            cpu_load_idle: data_list.cpu_idle,
            load_onem_avg: data_list.load_onem,
            load_fivem_avg: data_list.load_fivem,
            load_fifteenm_avg: data_list.load_fifteenm,
            cpu_temp: data_list.cpu_temp,
            disk_use: disk_data,
        }
    }

    // Here, the choice of date 60 seconds after Unix Epoch is totaly arbitrary, and is a simple way
    // to distinguish an error from an "uninitialized" or "empty" data. Similarly -1 and 0 for numbers.
    // For cpu_temp, it is just the minimum of f32 for error and absolute zero in celcius for "empty".
    pub fn new_error() -> StationData {
        return Self {
            no: -1,
            date: Duration::from_secs(60),
            uptime: -1,
            interface_data: "Error".to_string(),
            latency: -1.0,
            socket_stats: "Error".to_string(),
            memory_used: -1,
            memory_max: -1,
            memory_details: "Error".to_string(),
            swap_used: -1,
            swap_max: -1,
            swap_details: "Error".to_string(),
            cpu_load_user: -1.0,
            cpu_load_system: -1.0,
            cpu_load_idle: -1.0,
            load_onem_avg: -1.0,
            load_fivem_avg: -1.0,
            load_fifteenm_avg: -1.0,
            cpu_temp: f32::MIN,
            disk_use: "Error".to_string(),
        }
    }
    
    pub fn new_empty() -> StationData {
        Self {
            no: 0,
            date: Duration::from_secs(0),
            uptime: 0,
            interface_data: "Empty".to_string(),
            latency: 0.0,
            socket_stats: "Empty".to_string(),
            memory_used: 0,
            memory_max: 0,
            memory_details: "Empty".to_string(),
            swap_used: 0,
            swap_max: 0,
            swap_details: "Empty".to_string(),
            cpu_load_user: 0.0,
            cpu_load_system: 0.0,
            cpu_load_idle: 0.0,
            load_onem_avg: 0.0,
            load_fivem_avg: 0.0,
            load_fifteenm_avg: 0.0,
            cpu_temp: -273.15,
            disk_use: "Empty".to_string(),
        }
    }

    pub fn as_params(&self) -> Vec<Value> {
        // Station-provided timestamp
        let station_time = DateTime::<Utc>::from_timestamp(
            self.date.as_secs() as i64,
            self.date.subsec_nanos(),
        )
        .unwrap_or_else(|| DateTime::<Utc>::UNIX_EPOCH + chrono::Duration::seconds(60))
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S.%f %:z")  // <- format like last_alive
        .to_string();

        // Push time
        let push_time = Utc::now()
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S.%f %:z")
            .to_string();

        vec![
            self.no.into(),
            push_time.into(),
            station_time.into(),
            self.uptime.into(),
            self.interface_data.clone().into(),
            self.latency.into(),
            self.socket_stats.clone().into(),
            self.memory_used.into(),
            self.memory_max.into(),
            self.memory_details.clone().into(),
            self.swap_used.into(),
            self.swap_max.into(),
            self.swap_details.clone().into(),
            self.cpu_load_user.into(),
            self.cpu_load_system.into(),
            self.cpu_load_idle.into(),
            self.load_onem_avg.into(),
            self.load_fivem_avg.into(),
            self.load_fifteenm_avg.into(),
            self.cpu_temp.into(),
            self.disk_use.clone().into(),
        ]
    }
}
