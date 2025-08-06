use std::{fs::{self, File, OpenOptions}, io::{self, BufRead, ErrorKind}};

/// Reads the lines from the config file (used specifically for the config file (/etc/xbfisher/config) so writes config info if the file does not exist).
pub fn read_config() -> io::Result<io::Lines<io::BufReader<File>>> {
    let config_path = "/etc/xbfisher/config".to_string();
    fs::create_dir("/etc/xbfisher").unwrap_or_else(| error | {
        if error.kind() != ErrorKind::AlreadyExists {
            panic!("Could not create a directory for the config file. Please create the directory /etc/xbfisher. Error: {error}");
        }
    });
    let file = File::open(&config_path).unwrap_or_else(|error|{
        if error.kind() == ErrorKind::NotFound {
            let _ = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&config_path)
            .unwrap_or_else(|error|{
                panic!("Config file not found. Problem creating the config file: {}. Error: {error:?}", &config_path);
            });
            let info: String = "# To comment on this file, use a '#' at the start of the line.\n\
                                # The '#' in the middle of a line is not accepted as a comment!\n\
                                # These configurations are dynamic. They will take effect without restarting, in a maximum of 30 seconds.\n#\n\
                                # Initial Stations List:\n\
                                # To configure the initial station list, use the following pattern:\n\
                                # station=<StationNo> -<UserName> -<StationIP>\n\
                                # Example:\n\
                                # station=1 -central -10.8.0.101\n\n\
                                # These configurations are static. You will need to restart the program after changing.\n\
                                # Port for the server. Uses 2537 as default:\n\
                                server_port=2537\n\n\
                                # Interval between diagnostic data points in seconds:\n\
                                diag_data_interval=60\n\n\
                                # Interval between life checks in seconds:\n\
                                check_alive_interval=10\n".to_string();
            fs::write(&config_path, info).unwrap_or_else(|error| {panic!("Problem writing the use information to the config file. Error: {error}")});
            panic!("Couldn't find 'config' file. 'config' file created in /etc/xbfisher/config. Please configure before running again.");
        } else {
            panic!("Problem accessing the config file: {}. Error: {error:?}", &config_path)
        }
    });
    Ok(io::BufReader::new(file).lines())
}
