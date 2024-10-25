use socket2::{Domain, Protocol, Socket, Type};
use std::{io::Read, net::{IpAddr, SocketAddr}, time::{Duration, SystemTime}};

use crate::{pinging::{EchoReply, EchoRequest, IcmpV4, IpV4Packet, ICMP_HEADER_SIZE}, Error, math};

const KEY1: &str = "Suiladmellon";
const KEY2: &str = "suilad";
const IDENT: u16 = 4169;
const ECHO_REQUEST_BUFFER_SIZE: usize = ICMP_HEADER_SIZE + 24;

fn request_data(ipadrr: IpAddr, timeout: Option<Duration>) -> Result<Vec<u8>, Error>{
    let time_start = SystemTime::now();

    let timeout = match timeout {
        Some(timeout) => timeout,
        None => Duration::from_secs(4),
    };

    let station = SocketAddr::new(ipadrr, 0);
    
    // write the key
    let mut payload: [u8; 24] = [0; 24];
    let key_iter = KEY1.as_bytes().to_vec();
    for i in 0..KEY1.as_bytes().len() {
        payload[i] = key_iter[i]
    };

    println!("Payload Before Send: ");
    for i in 0..payload.len(){
        println!("{}", payload[i]);
    }

    let mut buffer = [0; ECHO_REQUEST_BUFFER_SIZE];

    let request = EchoRequest {
        ident: IDENT,
        seq_cnt: 1,
        payload: &payload,
    };

    if request.encode::<IcmpV4>(&mut buffer[..]).is_err() {
        return Err(Error::InternalError.into());
    }

    let mut socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;

    socket.set_ttl(64)?;
    socket.set_write_timeout(Some(timeout))?;
    socket.bind(&station.into())?;

    if (socket.send_to(&mut buffer, &station.into())?) == buffer.len(){
        println!("Success! FIX ME!!!!!!");
    }

    // loop until you get the data back or you reach the timeout
    let mut time_elapsed = Duration::from_secs(0);
    loop {
        println!("Hello!");
        let mut key = true;
        socket.set_read_timeout(Some(timeout-time_elapsed))?;

        let mut buffer: [u8; 2048] = [0; 2048];
        println!("Reading!");
        socket.read(&mut buffer)?;
        println!("Read!");

        let reply = {
            let ipv4_packet = match IpV4Packet::decode(&buffer) {
                Ok(packet) => packet,
                Err(_) => return Err(Error::DecodeV4Error.into()),
            };
            match EchoReply::decode::<IcmpV4>(ipv4_packet.data) {
                Ok(reply) => reply,
                Err(_) => continue,
            }
        };
        println!("Payload: ");
        for i in 0..32{
            println!("{}", reply.payload[i]);
        }

        // check for key
        let key_iter = KEY2.as_bytes().to_vec();
        for i in 0..KEY2.as_bytes().len(){
            println!("Key Check: {} =?= {}", reply.payload[i], key_iter[i]);
            if reply.payload[i] != key_iter[i]{
                key = false;
                break;
            };
        };

        // if the key and the identity are correct, return the data
        if reply.ident == request.ident && key {
            println!("Correct package, {key}");
            return Ok(reply.payload.to_vec());
        }

        // if ident is not correct check if timeout is over
        time_elapsed = match SystemTime::now().duration_since(time_start) {
            Ok(reply) => reply,
            Err(_) => return Err(Error::InternalError.into()),
        };
        if time_elapsed >= timeout {
            let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "Timeout occured");
            return Err(Error::IoError { error: (error) });
        }
    }

}

fn parse_data(data: Vec<u8>){
    let data = match String::from_utf8(data){
        Ok(b) => b,
        Err(error) => panic!("Error while parsing the return data: {error}")
    };

    let mut iter = data.split("...");
    while let Some(datum) = iter.next(){
        println!("Data: {}", data);
    }
}

pub fn get_stats(ipadrr: IpAddr, timeout: Option<Duration>) {
    let data: Vec<u8> = request_data(ipadrr, timeout).expect("Fix Me!");
    parse_data(data);
}