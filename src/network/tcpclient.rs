use std::io::prelude::*;
use std::net::TcpStream;
use std::str::FromStr;

use crate::Error;
use crate::station::Station;

/// TODO: READ THESE FROM A CONFIG FILE:
const PORT: &str = "2537";

pub enum CommandKind {
    ReqDiag,
    CheckAlive,
    ReqData,   
}

// TODO: Error handling for when TcpStream::connect fails.
pub fn req_comms(station: &Station, command: CommandKind) -> Result<Vec<String>, Error> {
    let mut stream = TcpStream::connect(format!("{}:{}", station.get_ip_address(), PORT)).unwrap();
    match command {
        // Requests Diagnosis Data from the station.
        CommandKind::ReqDiag => {
            // Sends the command along:
            let msg = String::from_str("REQDIAG").expect("Hardcoded.");
            let cmd: &[u8] = msg.as_bytes();
            let _ = stream.write(cmd);

            // Reads the response from the station:
            let response: &mut Vec<u8> = &mut Vec::new();
            let _ = stream.read_to_end(response);
            let response = String::from_utf8(response.to_vec());
            match response {
                Ok(diagdata) => {
                    // Some initial parsing...
                    let mut diagdata = diagdata.trim().trim_matches(char::from(0)).trim().to_string();
                    println!("NEW DATA:\n{:?}", diagdata);
                    if !diagdata.starts_with("StartDiag") || !diagdata.ends_with("ENDAll") {
                        return Err(Error::InvalidTCPReturn)
                    }
                    diagdata = diagdata.replace("StartDiag", "");
                    diagdata = diagdata.replace("ENDAll", "");
                    return Ok(diagdata.trim().split("End").map(|x| x.trim().to_string()).collect::<Vec<String>>());
                },
                Err(_) => return Err(Error::InvalidTCPCommunication),
            }
        },

        // Request a reply back from the station.
        CommandKind::CheckAlive => {
            let msg = String::from_str("CHECKAL").expect("Hardcoded.");
            let cmd: &[u8] = msg.as_bytes();
            let _ = stream.write(cmd);

            let response: &mut [u8; 2048] = &mut [0; 2048];
            let _ = stream.read(response);
            println!("{}", String::from_utf8(response.to_vec()).expect("Hardcoded."));
            todo!()
        },

        // Requests T2-T3 Data from the station.
        CommandKind::ReqData => todo!()
    };
}