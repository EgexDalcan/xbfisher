use std::{net::IpAddr, str::FromStr, time::Duration};
use xbfisher_client::send_data;

fn main() {
    send_data(IpAddr::from_str("10.8.0.6").expect("testing, will fix"), Some(Duration::from_secs(64))).unwrap();
}
