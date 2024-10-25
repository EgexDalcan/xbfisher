use xbfisher_server::commands::{ping_station_from_ip, start_data_from_list};

fn main() {
    ping_station_from_ip("10.8.0.110".into(), 5);
}