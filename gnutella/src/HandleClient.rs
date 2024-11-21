use crate::InitializeConn;
use crate::Messages;
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;

pub fn handle_requests(stream: &mut TcpStream) -> io::Result<()> {
    // Send a connection request
    let response = InitializeConn::request_conn(stream)?;

    // Check if server accepts connection
    if response.contains("200 OK") {
        println!("Connection successful");
        send_ping(stream, Messages::generate_desid(), 2, 0)?;
    } else {
        // Handle by requesting connection to other server
        println!("Connection unsuccessful");
    }
    Ok(())
}

pub fn send_ping(
    stream: &mut TcpStream,
    id: String,
    ttl: u8,
    hops: u8,
) -> Result<(), std::io::Error> {
    let ping_header = Messages::Header::new(id, Messages::Payload_type::Ping, ttl, hops, 0);

    let ping_header_bytes = ping_header.to_bytes();
    // println!("send ping: {:?}", ping_header_bytes);
    println!(
        "sending ping: {:?}",
        Messages::from_bytes(&ping_header_bytes)
    );
    stream.write_all(&ping_header_bytes)?;
    stream.flush()?;

    Ok(())
}
