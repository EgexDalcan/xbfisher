use xbfisher::{commands::start_data_from_db_list, database::{add_station_to_db, start_database}, parsing::parse_diag_data, req_comms, station::Station, CommandKind};

fn main() {
    let _ = start_database();
    add_station_to_db(&"2".to_string(), &"boromir".to_string(), &"10.8.0.110".to_string());
    start_data_from_db_list(&0.to_string());

    /* let args: Vec<String> = env::args().collect();
    if args.len() < 2 || &args[1] == &"-h".to_owned(){
        println!("XBFisher 1.0\nUsage: xbfisher [job] [options] <destination/parameters>
log:\n Can log the data from specified stations in the log file or in the parameters.
    -s: starts data logging from a specified ip address. Usage:\n    xbfisher log -s <user name> <ip_address> <interval>
    -l: starts logging from the hosts file into a csv document. Usage:\n    xbfisher log -l <interval>");
    } else if &args[1] == &"log".to_owned(){
        match args[2].as_str() {
            "-s" => {if args.len() == 5{start_data_from_ip(&args[3], &args[4], &args[5])}else{println!("log -s option requires an ip address and an interval.\nSee the output of 'xbfisher -h' for a summary of options.")}},
            "-l" => {if args.len() == 4{start_data_from_list(&args[3])}else{println!("log -l option requires an interval.\nSee the output of 'xbfisher -h' for a summary of options.")}},
            _ => println!("log option requires an argument.\nSee the output of 'xbfisher -h' for a summary of options.")
        }
    } else {
        println!("Unknown Command. See the output of 'xbfisher -h' for a summary of options.");
    } */
}