use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::station::Station;

// Diagnostic Data Parsing

#[derive(bincode::Decode)]
pub struct DiskData {
    name: String,
    used: u64,
    max: u64
}

impl DiskData {
    pub fn get_data(&self) -> String {
        format!("Disk: {} Use: {}/{} bytes.", self.name, self.used, self.max)
    }
}

#[derive(bincode::Decode)]
pub struct DiagData {
    pub date: u128,
    pub uptime: u64,
    pub networks: String,
    pub socket_stats: String,
    pub memory_used: u64,
    pub memory_max: u64,
    pub memory_details: String,
    pub swap_used: u64,
    pub swap_max: u64,
    pub swap_details: String,
    pub cpu_user: f32,
    pub cpu_nice: f32,
    pub cpu_system: f32,
    pub cpu_intr: f32,
    pub cpu_idle: f32,
    pub load_onem: f32,
    pub load_fivem: f32,
    pub load_fifteenm: f32,
    pub cpu_temp: f32,
    pub disks: Vec<DiskData>
}

// Config File Parsing

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

#[derive(Debug, Deserialize, Serialize)]
struct ConfigDataRaw {
    #[serde(default = "default_diag_interval")]
    diag_data_interval: u64,

    #[serde(default = "default_alive_interval")]
    check_alive_interval: u64,

    #[serde(default = "default_port")]
    server_port: String,

    #[serde(default = "default_db_loc")]
    database_location: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stations: Vec<StationConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct StationConfig {
    no: u8,
    name: String,
    ip: String,
}

fn default_diag_interval() -> u64 { 60 }
fn default_alive_interval() -> u64 { 10 }
fn default_port() -> String { "2537".to_string() }
fn default_db_loc() -> String { "/etc/xbfisher/station_database.db".to_string() }

impl ConfigData {
    pub fn from_toml_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(path);

        // If file doesn't exist, create it with default values
        if !path.exists() {
            let default_config = ConfigDataRaw {
                diag_data_interval: default_diag_interval(),
                check_alive_interval: default_alive_interval(),
                server_port: default_port(),
                database_location: default_db_loc(),
                stations: vec![],
            };

            let mut toml_str = String::new();
            toml_str.push_str("# Config file for XBFisher\n");
            toml_str.push_str("# Add each individual station under a new [[stations]]\n");
            toml_str.push_str("# Example:\n");
            toml_str.push_str("# \n");
            toml_str.push_str("# [[stations]]\n");
            toml_str.push_str("# no = 1\n");
            toml_str.push_str("# name = \"central\"\n");
            toml_str.push_str("# ip = \"127.0.0.1\"\n");
            toml_str.push_str("# \n");
            toml_str.push_str("# [[stations]]\n");
            toml_str.push_str("# no = 2\n");
            toml_str.push_str("# name = \"Galadriel\"\n");
            toml_str.push_str("# ip = \"127.0.0.1\"\n\n");

            toml_str += &toml::to_string_pretty(&default_config)?;

            toml_str.push_str("\n\n");

            fs::write(path, toml_str)?;
            println!("Created default config file at {}", path.display());
        }

        // Now read the file
        let content = fs::read_to_string(path)?;
        let raw: ConfigDataRaw = toml::from_str(&content)?;

        // Connect stations
        let mut stations = Vec::new();
        for s in raw.stations {
            match Station::connect_station(s.no, &s.name, &s.ip) {
                Ok(station) => stations.push(station),
                Err(e) =>{ eprintln!("Error: could not connect to station {}: {}", s.no, e); },
            }
        }

        Ok(ConfigData {
            station_vec: stations,
            diag_interval: raw.diag_data_interval,
            alive_interval: raw.check_alive_interval,
            port: raw.server_port,
            database_location: raw.database_location,
        })
    }
}
