use std::io::prelude::*;
use std::net::TcpStream;
use std::{thread, time};

pub fn request_conn(stream: &mut TcpStream) -> Result<String, std::io::Error> {
    // println!("inn");
    let mut message = "GNUTELLA CONNECT/0.4\n\n"; // gnutella connect request message
    let x = stream.write_all(message.as_bytes());
    stream.flush();

    // println!("{:?}",x);

    let mut buff = [0; 25];
    let bytes_read = stream.read(&mut buff)?;
    stream.flush()?;
    // thread::sleep(time::Duration::from_millis(1000));
    // stream.write_all(message.as_bytes());
    // stream.flush()?;

    let response = String::from_utf8_lossy(&buff[..bytes_read]).to_string();
    // thread::sleep(time::Duration::from_millis(1000));
    // let response = String::from_utf8_lossy(&buff[..bytes_read]).to_string();

    Ok(response)
}

pub fn accept_conn(mut stream: TcpStream) {
    let mut message = "GNUTELLA/0.4 200 OK\n\n"; // response to connect request if acceptable
    stream.write_all(message.as_bytes());
}

pub fn reject_conn(mut stream: TcpStream) {
    let mut message = "GNUTELLA/0.4 -1\n\n"; // response to connect request if not acceptable
    stream.write_all(message.as_bytes());
}
