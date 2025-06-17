use std::io::prelude::*;
use std::net::TcpStream;
use std::str::FromStr;

use crate::station::Station;

/// TODO: READ THESE FROM A CONFIG FILE:
const PORT: &str = "2537";

pub enum CommandKind {
    ReqDiag,
    CheckAlive,
    ReqData,   
}

pub fn send_command(station: Station, command: CommandKind) -> Vec<String>{
    match command {
        // Requests Diagnosis Data from the station.
        CommandKind::ReqDiag => {
            // Sends the command along:
            let mut stream = TcpStream::connect(format!("{}:{}", station.get_ip_address(), PORT)).unwrap();
            let msg = String::from_str("REQDIAG").expect("Hardcoded.");
            let cmd: &[u8] = msg.as_bytes();
            let _ = stream.write(cmd);

            // Reads the response from the station:
            let response: &mut [u8; 2048] = &mut [0; 2048];
            let _ = stream.read(response);
            println!("{}", String::from_utf8(response.to_vec()).expect("Hardcoded."));
        },

        CommandKind::CheckAlive=> todo!(),

        // Requests T2-T3 Data from the station.
        CommandKind::ReqData => todo!()
    };
    



    return vec!["God help me".into()];
}