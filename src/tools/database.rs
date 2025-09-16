use core::panic;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Error, Result};

use crate::{parsing::ConfigData, station::{Station, StationData}};

pub fn start_database(config_data: &ConfigData) -> Result<()> {
    let db = Arc::new(Mutex::new(Connection::open(config_data.get_db_loc()).unwrap()));

    let conn = db.lock().unwrap();

    conn.execute(
        "CREATE TABLE if not exists stations (
             station_number integer primary key,
             name text not null,
             ip_address text not null,
             last_alive text not null
         )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE if not exists station_data (
             station integer not null,
             unix_epoch text not null,
             date text not null,
             uptime text not null,
             network_data text not null,
             latency text not null,
             socket_stats text not null,
             memory text not null,
             memory_details text not null,
             swap text not null,
             swap_details text not null,
             cpu_load text not null,
             load_avg text not null,
             cpu_temp text not null,
             disk_use text not null,
             FOREIGN KEY(station) REFERENCES stations(station_number)
         )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE if not exists station_realtime_data (
             station integer primary key,
             unix_epoch text not null,
             date text not null,
             uptime text not null,
             network_data text not null,
             latency text not null,
             socket_stats text not null,
             memory text not null,
             memory_details text not null,
             swap text not null,
             swap_details text not null,
             cpu_load text not null,
             load_avg text not null,
             cpu_temp text not null,
             disk_use text not null,
             FOREIGN KEY(station) REFERENCES stations(station_number)
         )", 
        ()
    )?;
 
    for station in config_data.get_svec() {
        match add_station_to_db(&station.get_station_no().to_string(), &station.name, &station.ip_address, &station.last_alive.to_string(), config_data.get_db_loc()) {
            Ok(_) => (),
            // We panic here because it is the start of the program and the config file is obviously misconfigured.
            Err(error) => panic!("Failed to add the stations from the config file. Error: {}", error),
        }
    }
    Ok(())
}

pub fn add_station_to_db(stat_no: &String, stat_name: &String, stat_ip_addr: &String, stat_last_alive: &String, db_loc: &str) -> Result<()> {
    let db = Arc::new(Mutex::new(Connection::open(db_loc).unwrap()));

    let conn = db.lock().unwrap();
    
    conn.execute(
        "INSERT OR REPLACE INTO stations (station_number, name, ip_address, last_alive)
             VALUES ($1, $2, $3, $4)",
        (stat_no, stat_name, stat_ip_addr, stat_last_alive),
    )?;
    Ok(())
}

pub fn push_data_to_db(stat_data: &StationData, db_loc: &str) -> Result<()> {
    let db = Arc::new(Mutex::new(Connection::open(db_loc).unwrap()));

    let conn = db.lock().unwrap();

    conn.execute(
        "INSERT INTO station_data (station, unix_epoch, date, uptime, network_data, latency, socket_stats, memory, memory_details, swap, swap_details, cpu_load, load_avg, cpu_temp, disk_use)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
        stat_data.output_data(),
    )?;
    Ok(())
}

pub fn update_realtime_data(stat_data: &StationData, db_loc: &str) -> Result<()> {
    let db = Arc::new(Mutex::new(Connection::open(db_loc).unwrap()));

    let conn = db.lock().unwrap();

    conn.execute(
        "INSERT OR REPLACE INTO station_realtime_data (station, unix_epoch, date, uptime, network_data, latency, socket_stats, memory, memory_details, swap, swap_details, cpu_load, load_avg, cpu_temp, disk_use)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
        stat_data.output_data(),
    )?;
    Ok(())
}

pub fn db_update_last_alive(station: &Station, db_loc: &str) -> Result<()> {
    let db = Arc::new(Mutex::new(Connection::open(db_loc).unwrap()));

    let conn = db.lock().unwrap();

    conn.execute(
        "UPDATE stations
             SET last_alive = ($1)
             WHERE
                station_number = ($2)",
            (station.last_alive.to_string(), station.station_no)
    ).unwrap_or_else(|error| {panic!("Error while updating the last_alive column of station {} {error}", station.get_station_no())});
    Ok(())
}

pub fn db_update_stations_list(config_data: &ConfigData) {
    for station in config_data.get_svec() {
        match add_station_to_db(&station.get_station_no().to_string(), &station.name, &station.ip_address, &station.last_alive.to_string(), config_data.get_db_loc()) {
            Ok(_) => (),
            // We DO NOT panic here because it is not the start of the program anymore and we do not want to stop everything.
            Err(error) => eprintln!("Failed to add the stations from the config file. Error: {}", error),
        }
    }
}

pub fn get_stations(db_loc: &str) -> Result<Vec<String>, Error> { 
    let db = Arc::new(Mutex::new(Connection::open(db_loc).unwrap()));

    let conn = db.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT station_number, name, ip_address, last_alive
             FROM stations"
    )?;

    let mut rows = stmt.query([])?;

    
    let mut station_list = Vec::new();
    while let Some(row) = rows.next()? {
        station_list.push(format!("{} -{} -{}", row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    }

    Ok(station_list)
}