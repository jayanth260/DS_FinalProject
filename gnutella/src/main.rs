use std::io::prelude::*; 
use std::net::{TcpListener, TcpStream};
use std::env;
use std::thread;

mod HandleServent;
mod HandleClient;
pub mod InitializeConn;


fn main() -> std::io::Result<()> {    
    let args: Vec<String> = env::args().collect();
    let listener = TcpListener::bind(args[1].clone())?;

    println!("Server listening on {}", args[1]);

    let handle= thread::spawn(
        move|| {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    HandleServent::handle_connection(stream);
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });
    if args[2]!="-1"{
    let mut streamm = TcpStream::connect(args[2].clone())?; 
    HandleClient::handle_requests(streamm);
    
    }
    handle.join().unwrap();
    Ok(())

}

