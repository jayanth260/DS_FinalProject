use std::io::prelude::*; 
use std::net::{TcpStream};

use crate::InitializeConn;

pub fn handle_connection(mut stream: TcpStream)-> Result<(),std::io::Error>{
    let mut buff= [0;25];
    let bytes_read = stream.read(&mut buff)?;

    let response= String::from_utf8_lossy(&buff[..bytes_read]).to_string();
    
    if response.contains("CONNECT"){
        InitializeConn::Accept_conn(stream); // if connection is acceptable

    } 

    Ok(())
}