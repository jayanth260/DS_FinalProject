use std::io::prelude::*;
use std::io::{self, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::env;
use std::{thread, time};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
// use std::sync::Mutex;


mod HandleServent;
pub mod HandleClient;
pub mod InitializeConn;
pub mod Pong;
mod Messages;

pub static GLOBAL_PONG_PAYLOAD: Lazy<Mutex<Pong::Pong_Payload>> = Lazy::new(|| {
    Mutex::new(Pong::Pong_Payload {
        Port: String::new(),
        Ip: String::new(),
        Num_files: 0,
        Num_kb: 0
    })
});

pub struct PingPath {
    stream: Option<TcpStream>,
    id: String,
}

static PING_PATHS: Lazy<Mutex<Vec<PingPath>>> = Lazy::new(|| Mutex::new(Vec::new()));

impl PingPath {
    pub fn add_ping_path(stream: Option<TcpStream>, id: String) {
        let mut paths = PING_PATHS.lock().unwrap(); // Lock the mutex
        paths.push(PingPath { stream, id });
    }
    pub fn get_stream_by_id(id: &String) -> Option<TcpStream> {
        let paths = PING_PATHS.lock().unwrap(); // Lock the mutex
        for path in paths.iter() {
            if path.id == *id {
                // Attempt to clone the stream if it exists
                if let Some(ref stream) = path.stream {
                    return stream.try_clone().ok();
                }
            }
        }
        None // Return None if no match is found
    }
}

fn check_streams(streams: &mut Vec<Option<TcpStream>>) -> Result<(), std::io::Error> {
    // println!("here");
    // println!("{:?}",streams);
    let mut streams_dup: Vec<Option<TcpStream>> = streams.iter_mut()
    .map(|stream| {
        stream.as_mut().and_then(|s| s.try_clone().ok())
    })
    .collect();
    for stream_option in streams.iter_mut() {
        if let Some(stream) = stream_option {
            // Set non-blocking mode
            stream.set_nonblocking(true)?;
            
            let mut buff = [0; 1024];
            match stream.read(&mut buff) {
                Ok(bytes_read) if bytes_read > 0 => {
                    // println!("{:?}",buff);
                    if let Ok(stream_clone) = stream.try_clone() {
                        // Set blocking mode back for handling
                        stream_clone.set_nonblocking(false)?;
                        if let Err(e) = HandleServent::handle_connection(&mut streams_dup,stream_clone,bytes_read,&buff) {
                        //     eprintln!("Error handling connection: {}", e);
                        }
                    }
                }
                Ok(_) => {
                    // Zero bytes read - connection closed by peer
                    println!("mkl");
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

    if let Ok(mut payload) = GLOBAL_PONG_PAYLOAD.lock() {
        // Split first argument into IP and port
        let parts: Vec<&str> = args[1].split(':').collect();
        payload.Ip = parts[0].to_string();
        payload.Port = parts[1].to_string();
    
        // Convert string arguments to numbers
        payload.Num_files = args[3].parse().unwrap_or(0);
        payload.Num_kb = args[4].parse().unwrap_or(0);
    }

    let listener = Arc::new(listener);
    let handle_listener = Arc::clone(&listener);
    
    let streams: Arc<Mutex<Vec<Option<TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));
    let streams_clone1 = Arc::clone(&streams);
    let streams_clone2 = Arc::clone(&streams);
    
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
    
    
    let handle = thread::spawn(move || {
        if args[2] != "-1" {
            match TcpStream::connect(args[2].clone()) {
                Ok(mut stream) => {
                    // let stream_dup=stream.try_clone();
                    HandleClient::handle_requests(&mut stream);
                    if let Ok(mut streams) = streams_clone2.lock() {
                        streams.push(Some(stream));
                    }
                    
                    
                }
                Err(e) => {
                    eprintln!("Failed to connect to {}: {}", args[2], e);
                }
            }
        }
        

        
    });
    handle.join().unwrap();
    // Handle incoming connections
    for stream in handle_listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {:?}", stream);
                if let Ok(mut streams) = streams_clone1.lock() {
                    streams.push(Some(stream));
                }
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    

    
    
    checker_handle.join().unwrap();
    
    println!("Exiting...");
    Ok(())
}