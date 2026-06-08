use std::{sync::{Arc, Mutex}, thread, time::Duration};

use xbfisher::{commands::{start_diag_data_from_db, update_last_alives, update_program}, database::start_database, parsing::ConfigData};

fn main() {
    let config_data = Arc::new(Mutex::new(ConfigData::from_toml_file("/etc/xbfisher/config.toml")
    .unwrap_or_else( | error| { panic!("The config file is misconfigured: {}", error) })));

    {
        let cfg = config_data.lock().unwrap();
        match start_database(&cfg) {
            Ok(_) => (),
            // We panic here because if we cannot start the database everything should break.
            Err(error) => panic!("Failed to start the database. Error: {}", error)
        }
    }

    // We duplicate the config files to move into threads.
    let cnfg = Arc::clone(&config_data);

    // Vector to hold all the threads.
    let mut thread_vec = Vec::new();

    // The thread for collecting diagnostic data.
    thread_vec.push(thread::spawn( move || {
        loop {
            update_program(&cnfg);

            let interval = {
                let cfg = cnfg.lock().unwrap();
                Duration::from_secs(cfg.get_diag_interval())
            };

            {
                let cfg = cnfg.lock().unwrap();
                start_diag_data_from_db(&cfg);
            }
            
            thread::sleep(interval);
        }
    }));

    // The thread for checking for alive.
    thread_vec.push(thread::spawn( move || {
        loop {
            update_program(&config_data);

            let interval = {
                let cfg = config_data.lock().unwrap();
                Duration::from_secs(cfg.get_alive_interval())
            };

            {
                let cfg = config_data.lock().unwrap();
                update_last_alives(&cfg);
            }
            
            thread::sleep(interval);
        }
    }));

    // Wait for the threads to join, since they are infinite loops, will stop only when the user terminates.
    for i in thread_vec {
        i.join().expect("Could not join a thread");
    };
}