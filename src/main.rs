use std::{sync::Arc, thread};

use xbfisher::{commands::{start_diag_data_from_db, update_last_alives}, database::start_database, filecontrol::read_config, parsing::parse_config_file};

fn main() {
    let config_data = Arc::new(parse_config_file(read_config()));

    match start_database(&config_data) {
        Ok(_) => (),
        // We panic here because if we cannot start the database everything should break.
        Err(error) => panic!("Failed to start the database. Error: {}", error)
    }

    let cnfg = Arc::clone(&config_data);
    let mut thread_vec = Vec::new();

    thread_vec.push(thread::spawn( move || {
        start_diag_data_from_db(&cnfg);
    }));

    thread_vec.push(thread::spawn( move || {
        update_last_alives(&config_data);
    }));

    for i in thread_vec {
        i.join().unwrap()
    };
}