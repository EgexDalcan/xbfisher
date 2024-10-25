use std::{io::Read, net::{IpAddr, SocketAddr}, thread::{self, Thread}, time::Duration};
use chrono::{Local, Timelike};
use socket2::{Domain, Protocol, Socket, Type};
use systemstat::{Platform, System};

use crate::Error;
use super::{icmp::{EchoReply, EchoRequest, IcmpV4}, ipv4::IpV4Packet};

const KEY1: &str = "Suiladmellon";
const KEY2: &str = "suilad";
const IDENT: u16 = 4169;

#[derive(serde::Serialize)]
pub struct DataRow{
    time: u8,
    cpu_temperature: u8,
    cpu_load: u8,
}

pub fn send_data(server_ip: IpAddr, timeout: Option<Duration>) -> Result<usize, Error>{
    let station = SocketAddr::new(server_ip, 0);
    let mut socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
    socket.set_ttl(64)?;
    socket.set_write_timeout(timeout)?;
    println!("hello1!");

    loop{
        println!("hello!");
        let mut key = true;
        let mut buffer: [u8; 2048] = [0; 2048];
        println!("waiting for buffer...");
        socket.read(&mut buffer)?;
        println!("buffer start:");
        for i in 0..25 {
            println!("{}", buffer[i]);
        };
        println!("buffer end.");

        let request = {
            let ipv4_packet = match IpV4Packet::decode(&buffer) {
                Ok(packet) => packet,
                Err(_) => return Err(Error::DecodeV4Error.into()),
            };
            match EchoReply::decode::<IcmpV4>(ipv4_packet.data) {
                Ok(request) => request,
                Err(error) => {for i in 0..ipv4_packet.data.len(){println!("{}", ipv4_packet.data[i])};panic!("Error during decoding: {error}")},
            }
        };

        // Check for key
        let key_iter = KEY1.as_bytes().to_vec();
        for i in 0..KEY1.as_bytes().len(){
            println!("{} =?= {}", request.payload[i], key_iter[i]);
            if request.payload[i] != key_iter[i]{
                key = false;
                break;
            }
        };

        println!("identity: {}", request.ident);
        println!("key: {}", key);

        // check if the identity and the key are correct
        if request.ident == IDENT && key {
            let data_load = collect_data();
            println!("data load: {}", data_load);
            println!("byte line start:");
            for i in 0..data_load.as_bytes().len(){
                println!("{}", data_load.as_bytes()[i]);
            }
            println!("byte line end");
            println!("byte to string: \n {}", String::from_utf8(data_load.as_bytes().to_vec()).unwrap());

            // write the key
            let key_iter = KEY2.as_bytes().to_vec();
            let mut payload: [u8; 2048] = [0; 2048];
            for i in 0..KEY2.as_bytes().len(){
                payload[i] = key_iter[i];
                println!("{}", key_iter[i]);
            };

            // write the data
            for i in 0..data_load.as_bytes().len(){
                payload[i + KEY2.as_bytes().len()] = data_load.as_bytes()[i];
                println!("{}", payload[i + KEY2.as_bytes().len()]);
            }

            let reply = EchoRequest{
                ident: IDENT,
                seq_cnt: 1,
                payload: &payload,
            };

            if reply.encode::<IcmpV4>(&mut buffer[..]).is_err() {
                return Err(Error::InternalError.into());
            }
            thread::sleep(Duration::from_secs(1));
            socket.send_to(&mut buffer, &station.into())?;
        };
    }
}

fn collect_data() -> String {
    let date: chrono::DateTime<Local> = chrono::offset::Local::now();
    format!("{}...{}...{}",
        format!("{}:{}", date.hour(), date.minute()),
        match get_current_temperature(){
            Ok(a) => a.to_string(),
            Err(error) => format!("Error: {}", error).to_string()
        },
        match get_current_cpu_load(){
            Ok(a) => format!("{}%", a),
            Err(error) => format!("Error: {}", error).to_string()
        })
    }

fn get_current_temperature() -> Result<f32, Error> {
    let sys = System::new();
    match sys.cpu_temp() {
        Ok(cpu_temp) => Ok(cpu_temp),
        Err(x) => Err(Error::IoError { error: (x) }),
    }
}

fn get_current_cpu_load() -> Result<f32, Error>{
    let sys = System::new();
    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            thread::sleep(Duration::from_secs(1));
            let cpu = cpu.done().unwrap();
            Ok(100.0 - (cpu.idle * 100.0))
        },
        Err(x) => Err(Error::IoError { error: (x) })
    }
}