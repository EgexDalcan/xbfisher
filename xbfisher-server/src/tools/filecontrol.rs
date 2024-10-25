use std::{fs::{self, File, OpenOptions}, io::{self, BufRead, ErrorKind}, thread, time::Duration};

use chrono::{Datelike, Local};
use csv::WriterBuilder;

use crate::stations::station;

/// Reads the lines from a given file (used specifically for the config file (./hosts) so writes config info if the file does not exist).
pub fn read_lines(filename: String) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(&filename).unwrap_or_else(|error|{
        if error.kind() == ErrorKind::NotFound{
            let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&filename)
            .unwrap_or_else(|error|{
                panic!("Problem creating the hosts file: {filename}. Error: {error:?}");
            });
            let info: String = "# To configure the station list use the following pattern:\n# StationNo -StationIP\n# Example:\n# 1 -10.8.0.101".into();
            fs::write(filename, info).unwrap_or_else(|error| {panic!("Problem writing to the hosts file. Error: {error}")});
            panic!("hosts config file created");
            file
        } else {
            panic!("Problem accessing the hosts file: {filename}. Error: {error:?}")
        }
    });
    Ok(io::BufReader::new(file).lines())
}

/// Gathers data using gather_data_set() and then parses and writes it on a .csv file.
pub fn write_data(station: station::Station, interval: u64){
    loop{
        let data = station.gather_data_set();
        let mut header: bool = false;
        let date: chrono::DateTime<Local> = chrono::offset::Local::now();
        let file_name = format!("station_{}_date_{}.csv", station.station_no, (format!("{}_{}_{}", date.month(), date.day(), date.year())));
        // Open file, if no file, create one.
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&file_name)
            .unwrap_or_else(| error | {
                if error.kind() == ErrorKind::NotFound {
                    header = true;
                    OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(&file_name)
                        .unwrap_or_else(| error | {
                            panic!("Problem creating the file: {file_name}. Error: {error:?}");
                            })
                    } else {
                        panic!("Problem opening the file: {file_name}. Error: {error:?}");
                    }
            });
        // Write.
        let mut wtr = WriterBuilder::new()
            .has_headers(header)
            .from_writer(file);
        wtr.serialize(data).unwrap();
        thread::sleep(Duration::from_secs(interval));
    }
}

pub fn write_list_data(svec: &Vec<station::Station>, interval: u64){
    let mut datavec: Vec<station::DataRow> = vec![];
    for i in svec{
        datavec.push(i.gather_data_set());
    }
    let mut header: bool = false;
        let date: chrono::DateTime<Local> = chrono::offset::Local::now();
        let file_name = format!("station_list_date_{}.csv", (format!("{}_{}_{}", date.month(), date.day(), date.year())));
        // Open file, if no file, create one.
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&file_name)
            .unwrap_or_else(| error | {
                if error.kind() == ErrorKind::NotFound {
                    header = true;
                    OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(&file_name)
                        .unwrap_or_else(| error | {
                            panic!("Problem creating the file: {error:?}");
                            })
                    } else {
                        panic!("Problem opening the file: {error:?}");
                    }
            });
        // Write.
        let mut wtr = WriterBuilder::new()
            .has_headers(header)
            .from_writer(file);
        wtr.serialize(datavec).unwrap();
        thread::sleep(Duration::from_secs(interval));
}