use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Error, Result};

use crate::station::StationData;

pub fn start_database() -> Result<()> {
    let db = Arc::new(Mutex::new(Connection::open("station_data.db3").unwrap()));

    let conn = db.lock().unwrap();

    conn.execute(
        "create table if not exists stations (
             station_number integer primary key,
             station_name text not null unique,
             station_ip_address text not null unique
         )",
        (),
    )?;

    conn.execute(
        "create table if not exists station_data (
             station_number integer not null,
             station_name text not null,
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
             cpu_temp text not null
         )",
        (),
    )?;
    
    // TODO: Remove, exists for testing reasons!
    let _ = add_station_to_db(&"1".to_string(), &"TEST".to_string(), &"127.0.0.1".to_string());

    Ok(())
}

pub fn add_station_to_db(stat_no: &String, stat_name: &String, stat_ip_addr: &String) -> Result<()> {
    let db = Arc::new(Mutex::new(Connection::open("station_data.db3").unwrap()));

    let conn = db.lock().unwrap();
    
    conn.execute(
        "INSERT INTO stations (station_number, station_name, station_ip_address)
             VALUES ($1, $2, $3)",
        (stat_no, stat_name, stat_ip_addr),
    )?;
    Ok(())
}

pub fn push_data_to_db(stat_data: &StationData) -> Result<()> {
    let db = Arc::new(Mutex::new(Connection::open("station_data.db3").unwrap()));

    let conn = db.lock().unwrap();

    conn.execute(
        "INSERT INTO station_data (station_number, station_name, date, uptime, network_data, latency, socket_stats, memory, memory_details, swap, swap_details, cpu_load, load_avg, cpu_temp)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        stat_data.output_data(),
    )?;

    Ok(())
}

pub fn get_stations() -> Result<Vec<String>, Error> { 
    let db = Arc::new(Mutex::new(Connection::open("station_data.db3").unwrap()));

    let conn = db.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT station_number, station_name, station_ip_address
             FROM stations"
    )?;

    let mut rows = stmt.query([])?;

    
    let mut station_list = Vec::new();
    while let Some(row) = rows.next()? {
        station_list.push(format!("{} -{} -{}", row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    }

    Ok(station_list)
}