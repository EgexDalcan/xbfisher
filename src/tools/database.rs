use std::sync::{Arc, Mutex};

use rusqlite::{Connection, Result};

pub fn start_database() -> Result<()> {
    let db = Arc::new(Mutex::new(Connection::open("station_data.db").unwrap()));

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
             station_number integer primary key,
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

    Ok(())
}
