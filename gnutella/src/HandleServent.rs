use std::io::prelude::*;
use std::net::TcpStream;
use crate::InitializeConn;
use crate::Messages;

pub fn handle_connection(mut stream: TcpStream,bytes_read: usize, buff: &[u8] ) -> Result<(), std::io::Error> {
    println!("in");
    let mut buffer = buff.clone();
    // let bytes_read = stream.read(&mut buff)?;
    let mut response = String::from_utf8_lossy(&buff[..bytes_read]).to_string();
    println!("{}", response);
    
    if response.contains("CONNECT") {
        InitializeConn::accept_conn(stream); // if connection is acceptable
    }
    else{
        let response1 : Messages::Header = Messages::from_bytes(buffer).expect("REASON");
        println!("{}",response1.get_descriptor_id());
    }

    
    Ok(())
}
