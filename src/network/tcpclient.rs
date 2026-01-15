use std::io::prelude::*;
use std::net::TcpStream;
use std::str::FromStr;

use bincode::config::Configuration;

use crate::Error;
use crate::parsing::DiagData;
use crate::station::Station;

pub enum CommandKind {
    ReqDiag,
    CheckAlive,
    ReqData,
}

pub enum ReturnKind {
    DiagRet(DiagData),
    AliveRet,
    Err(Error)
}

pub fn req_comms(station: &Station, command: CommandKind, port: &str) -> ReturnKind {
    let mut stream = match TcpStream::connect(format!("{}:{}", station.get_ip_address(), port)) {
        Ok(stream) => stream,
        Err(error) => return ReturnKind::Err(Error::TCPStreamError { error: error }),
    };
    match command {
        // Requests Diagnosis Data from the station.
        CommandKind::ReqDiag => {
            // Sends the command along:
            let msg = String::from_str("REQDIAG").expect("Hardcoded.");
            let cmd: &[u8] = msg.as_bytes();
            let _ = stream.write(cmd);

            // Reads the response from the station:
            let config = bincode::config::standard();
            match bincode::decode_from_std_read::<DiagData, Configuration, TcpStream>(&mut stream, config) {
                Ok(diagdata) => {
                    return ReturnKind::DiagRet(diagdata)
                },
                Err(_) => return ReturnKind::Err(Error::InvalidTCPCommunication),
            }
        },

        // Request a reply back from the station.
        CommandKind::CheckAlive => {
            let msg = String::from_str("CHECKAL").expect("Hardcoded.");
            let cmd: &[u8] = msg.as_bytes();
            let _ = stream.write(cmd);

            let response: &mut Vec<u8> = &mut Vec::new();
            let _ = stream.read_to_end(response);
            let response = String::from_utf8(response.to_vec());
            match response {
                Ok(resp) => {
                    let response = resp.trim_matches(char::from(0)).trim().to_string();
                    if response.starts_with("ALIVE") && response.ends_with("ALIVE") {
                        return ReturnKind::AliveRet
                    }
                    return ReturnKind::Err(Error::InvalidTCPReturn);
                },
                Err(_) => return ReturnKind::Err(Error::InvalidTCPCommunication),
            }
        },

        // Requests T2-T3 Data from the station.
        CommandKind::ReqData => todo!()
    };
}