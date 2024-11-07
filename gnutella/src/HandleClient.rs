use std::io::prelude::*; 
use std::net::{TcpStream};

use crate::InitializeConn;

pub fn handle_requests(mut stream: TcpStream){
    // send a connection request
    let response= InitializeConn::Request_conn(stream);
    // check if server accepts connection
    if response.expect("REASON").contains("200 OK"){
        println!("Conection successful");
    }
    else{
        // handle by requesting connection to other server
        println!("Connection Unsccessful");
    }

}