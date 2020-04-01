extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use clap::{App, Arg};
use regex::Regex;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::str::from_utf8;

fn main() {
    let matches = App::new("Fart Proxy")
        .version("0.1.0")
        .author("Pavlo Hlushchenko <pavlo.hlushchenko@gmail.com>")
        .about("Burp-like proxy")
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .value_name("ADDRESS")
                .help("Sets a custom address and port to listen on")
                .takes_value(true)
                .default_value("127.0.0.1:7878"),
        )
        .get_matches();
    let address = matches.value_of("listen").unwrap();
    let listener = TcpListener::bind(address).expect(&format!("Couldn't bind to {}", address));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    match stream.read(&mut buffer) {
        Ok(_) => {}
        Err(e) => {
            println!("Error when reading request: {}", e);
        }
    }
    // This is done in order not to compile the regex pattern on every call of handle_connection
    lazy_static! {
        static ref RE: Regex = Regex::new(r"Host: (?P<host>[0-9a-zA-Z\-\.:]+)").unwrap();
    }
    let caps = RE.captures(from_utf8(&buffer).unwrap()).unwrap();
    let mut host = String::from(caps.name("host").unwrap().as_str());
    if !host.contains(":") {
        host.push_str(":80");
    }

    match TcpStream::connect(&host) {
        Ok(mut server_stream) => {
            server_stream.write_all(&buffer).unwrap();
            let mut data = [0 as u8; 64000];
            match server_stream.read(&mut data) {
                Ok(_) => {
                    stream.write_all(&data).unwrap();
                }
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to connect to {}: {}", &host, e);
        }
    }

    stream.flush().unwrap();
}
