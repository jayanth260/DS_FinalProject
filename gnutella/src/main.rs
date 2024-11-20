use std::io::prelude::*;
use std::io::{self, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::env;
use std::{thread, time};
use std::sync::{Arc, Mutex};
mod HandleServent;
mod HandleClient;
pub mod InitializeConn;
mod Messages;

fn check_streams(streams: &mut Vec<Option<TcpStream>>) -> Result<(), std::io::Error> {
    println!("here");
    for stream_option in streams.iter_mut() {
        if let Some(stream) = stream_option {
            // Set non-blocking mode
            stream.set_nonblocking(true)?;
            
            let mut buff = [0; 1024];
            match stream.read(&mut buff) {
                Ok(bytes_read) if bytes_read > 0 => {
                    println!("{:?}",buff);
                    if let Ok(stream_clone) = stream.try_clone() {
                        // Set blocking mode back for handling
                        stream_clone.set_nonblocking(false)?;
                        if let Err(e) = HandleServent::handle_connection(stream_clone,bytes_read,&buff) {
                        //     eprintln!("Error handling connection: {}", e);
                        }
                    }
                }
                Ok(_) => {
                    // Zero bytes read - connection closed by peer
                    *stream_option = None;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No data available right now, continue to next stream
                    continue;
                }
                Err(e) => {
                    // Real error occurred
                    eprintln!("Error reading from stream: {}", e);
                    *stream_option = None;
                }
            }
        }
    }
    
    // Clean up disconnected streams
    streams.retain(|stream| stream.is_some());
    
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let listener = TcpListener::bind(args[1].clone())?;
    println!("Server listening on {}", args[1]);
    
    let listener = Arc::new(listener);
    let handle_listener = Arc::clone(&listener);
    
    let streams: Arc<Mutex<Vec<Option<TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));
    let streams_clone = Arc::clone(&streams);
    
    // Spawn stream checker thread
    let checker_handle = {
        let streams = Arc::clone(&streams);
        thread::spawn(move || {
            loop {
                thread::sleep(time::Duration::from_secs(1));
                if let Ok(mut streams) = streams.lock() {
                    if let Err(e) = check_streams(&mut streams) {
                        eprintln!("Error checking streams: {}", e);
                    }
                }
            }
        })
    };
    
    // Handle incoming connections
    let handle = thread::spawn(move || {
        for stream in handle_listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {:?}", stream);
                    if let Ok(mut streams) = streams_clone.lock() {
                        streams.push(Some(stream));
                    }
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    if args[2] != "-1" {
        match TcpStream::connect(args[2].clone()) {
            Ok(stream) => {
                HandleClient::handle_requests(stream)?;
            }
            Err(e) => {
                eprintln!("Failed to connect to {}: {}", args[2], e);
            }
        }
    }

    handle.join().unwrap();
    checker_handle.join().unwrap();
    
    println!("Exiting...");
    Ok(())
}