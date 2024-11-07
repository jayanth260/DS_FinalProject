use std::io::prelude::*; 
use std::net:: TcpStream;

pub fn Request_conn(mut stream: TcpStream) ->Result<String,std::io::Error>  {
    
    let mut message = "GNUTELLA CONNECT/0.4\n\n"; // gnutella connect request message
    stream.write_all(message.as_bytes());
    let mut buff=[0;25];
    let bytes_read= stream.read(&mut buff)?;

    let response = String::from_utf8_lossy(&buff[..bytes_read]).to_string();
    Ok(response)

}

pub fn Accept_conn(mut stream: TcpStream){

    let mut message = "GNUTELLA/0.6 200 OK\n\n"; // response to connect request if acceptable
    stream.write_all(message.as_bytes());

}

pub fn Reject_conn(mut stream: TcpStream){

    let mut message="GNUTELLA/0.6 -1\n\n"; // response to connect request if not acceptable
    stream.write_all(message.as_bytes());
}