use std::io::prelude::*;
use std::net::TcpStream;
use crate::InitializeConn;
use crate::Messages;
use std::io;

pub fn handle_requests(mut stream: TcpStream) -> io::Result<()> {
    // Send a connection request
    let response = InitializeConn::request_conn(&mut stream)?;
 
    
    // Check if server accepts connection
    if response.contains("200 OK") {
        println!("Connection successful");
        send_ping(&mut stream)?;  
    } else {
        // Handle by requesting connection to other server
        println!("Connection unsuccessful");
    }
    Ok(())
}

fn send_ping(stream: &mut TcpStream) -> Result<(),std::io::Error>  {
    
    let ping_header = Messages::Header::new(
        Messages::generate_desid(),
        Messages::Payload_type::Ping,
        8,
        0,
        0
    );
    
    let ping_header_bytes = ping_header.to_bytes();
    println!("{:?}", ping_header_bytes);
    println!("{:?}",Messages::from_bytes(&ping_header_bytes));
    stream.write_all(&ping_header_bytes)?;
    stream.flush()?; 
    
    Ok(())
}