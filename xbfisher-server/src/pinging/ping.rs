use std::io::Read;
use std::net::{IpAddr, SocketAddr};
use std::ops::Div;
use std::time::{Duration,SystemTime};

use rand::random;
use socket2::{Domain, Protocol, Socket, Type};

use crate::errors::Error;
use crate::pinging::{EchoReply, EchoRequest, IcmpV4, IcmpV6, IpV4Packet, ICMP_HEADER_SIZE};
use crate::stations::station::Station;
use crate::tools::math;

const TOKEN_SIZE: usize = 32;
const ECHO_REQUEST_BUFFER_SIZE: usize = ICMP_HEADER_SIZE + TOKEN_SIZE;
type Token = [u8; TOKEN_SIZE];

pub fn ping(
    addr: IpAddr,
    timeout: Option<Duration>,
    ttl: Option<u32>,
    ident: Option<u16>,
    seq_cnt: Option<u16>,
    payload: Option<&Token>,
) -> Result<Duration, Error> {
    let time_start = SystemTime::now();

    let timeout = match timeout {
        Some(timeout) => timeout,
        None => Duration::from_secs(4),
    };

    let dest = SocketAddr::new(addr, 0);
    let mut buffer = [0; ECHO_REQUEST_BUFFER_SIZE];

    let default_payload: &Token = &random();

    let request = EchoRequest {
        ident: ident.unwrap_or(random()),
        seq_cnt: seq_cnt.unwrap_or(1),
        payload: payload.unwrap_or(default_payload),
    };

    let mut socket = if dest.is_ipv4() {
        if request.encode::<IcmpV4>(&mut buffer[..]).is_err() {
            return Err(Error::InternalError.into());
        }
        Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?
    } else {
        if request.encode::<IcmpV6>(&mut buffer[..]).is_err() {
            return Err(Error::InternalError.into());
        }
        Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6))?
    };

    if dest.is_ipv4() {
        socket.set_ttl(ttl.unwrap_or(64))?;
    } else {
        socket.set_unicast_hops_v6(ttl.unwrap_or(64))?;
    }

    socket.set_write_timeout(Some(timeout))?;

    socket.send_to(&mut buffer, &dest.into())?;

    // loop until either an echo with correct ident was received or timeout is over
    let mut time_elapsed = Duration::from_secs(0);
    loop {
        socket.set_read_timeout(Some(timeout - time_elapsed))?;

        let mut buffer: [u8; 2048] = [0; 2048];
        socket.read(&mut buffer)?;

        let reply = if dest.is_ipv4() {
            let ipv4_packet = match IpV4Packet::decode(&buffer) {
                Ok(packet) => packet,
                Err(_) => return Err(Error::DecodeV4Error.into()),
            };
            match EchoReply::decode::<IcmpV4>(ipv4_packet.data) {
                Ok(reply) => reply,
                Err(_) => continue,
            }
        } else {
            match EchoReply::decode::<IcmpV6>(&buffer) {
                Ok(reply) => reply,
                Err(_) => continue,
            }
        };

        if reply.ident == request.ident {
            time_elapsed = match SystemTime::now().duration_since(time_start) {
                Ok(reply) => reply,
                Err(_) => return Err(Error::InternalError.into()),
            };
            // received correct ident
            return Ok(time_elapsed);
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

pub fn ping_station(station: &Station, ping_count: u16) -> Vec<f32>{
    let time_start = SystemTime::now();
    let addr = station.get_ip_address().parse().expect("If we are able to create a Station type, the IPAdress must be correct.");
    let timeout = Duration::from_secs(2);
    let mut success_counter: u16 = 0;
    let mut fail_counter: u16 = 0;
    let mut latency: Vec<f32> = Vec::new();
    let ttl: u32 = 64;
    let interval: u64 = 1;
    while success_counter + fail_counter < ping_count {
        match ping(
            addr,
            Some(timeout),
            Some(ttl),
            Some(3),
            Some(5),
            Some(&random()),
        ){
            Ok(a) => {
                latency.push(a.as_micros() as f32 / 1000.0);
                println!("32 bytes from {addr}: ttl={} time={} ms", ttl, a.as_micros() as f32 /1000.0);
                success_counter = success_counter + 1;
            },
            Err(error) => {
                println!("Problem during pinging Station {}. Error: {error}",station.get_station_no());
                fail_counter = fail_counter + 1;
                continue;
            },
        }
        std::thread::sleep(Duration::from_secs(interval));
    }
    let viter = latency.iter();
    let avg = vec_mean(&latency);
    let mdev = vec_mdev(&latency);
    println!("{ping_count} packets transmitted, {success_counter} recieved, {}% packet loss, time {} ms", (fail_counter/ping_count) as f32 *100.0, SystemTime::now().duration_since(time_start).unwrap_or_else(|_|{Duration::from_secs(0)}).as_micros() as f32 / 1000.0);
    println!("min/avg/max/mdev = {}/{:.*}/{}/{:.*} ms", viter.clone().min_by(|x, y| x.partial_cmp(&y).unwrap()).unwrap_or(&0.0), math::n_decimals(avg, 4), avg, viter.clone().max_by(|x, y| x.partial_cmp(&y).unwrap()).unwrap_or(&0.0), math::n_decimals(mdev, 4), mdev);
    latency
}

/// Calculates the mean of the input vector.
pub fn vec_mean(v: &Vec<f32>) -> f32{
    let viter = v.iter();
    viter.clone().sum::<f32>() as f32 / viter.clone().len() as f32
}

/// Calculates the standard deviation of the input vector.
pub fn vec_mdev(v: &Vec<f32>) -> f32{
    let avg = vec_mean(v);
    let mut sum= 0.0;
    for i in v{
        sum = sum + (i - avg).powi(2);
    };
    sum.div(v.len() as f32).sqrt()
}

pub fn ping_station_silent(station: &Station, ping_count: u16) -> Vec<f32>{
    let addr = station.get_ip_address().parse().expect("If we are able to create a Station type, the IPAdress must be correct.");
    let timeout = Duration::from_secs(2);
    let mut success_counter: u16 = 0;
    let mut fail_counter: u16 = 0;
    let mut latency = Vec::new();
    let ttl: u32 = 64;
    let interval: u64 = 1;
    while success_counter + fail_counter < ping_count {
        match ping(
            addr,
            Some(timeout),
            Some(ttl),
            Some(3),
            Some(5),
            Some(&random()),
        ){
            Ok(a) => {
                latency.push(a.as_micros() as f32 / 1000.0);
                success_counter = success_counter + 1;
            },
            Err(_) => {
                fail_counter = fail_counter + 1;
                continue;
            },
        }
        std::thread::sleep(Duration::from_secs(interval));
    }
    latency
}

