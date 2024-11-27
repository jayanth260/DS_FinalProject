use once_cell::sync::Lazy;
use std::env;
use std::io::prelude::*;
use std::io::{self, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use std::collections::HashMap;
use lazy_static::lazy_static;
// use std::sync::Mutex;
use uuid::Uuid;
 
pub mod HandleFiles;
mod HandleServent;
pub mod HandleClient;
pub mod InitializeConn;
mod Messages;
pub mod Pong;
pub mod Query;
pub mod QueryHit;

lazy_static! {
    pub static ref GLOBAL_QUERYHIT_PAYLOADS: Mutex<HashMap<String, Vec<QueryHit::QueryHit_Payload>>> = 
        Mutex::new(HashMap::new());
}

pub static SERVENT_ID:Lazy<Uuid> = Lazy::new(|| {
    Uuid::new_v4()
});

// lazy_static! {
//     #[derive(Debug)]
//     pub static ref total_count: Mutex<u32> = Mutex::new(0);
//     pub static ref query_hit: Mutex<u32> = Mutex::new(0);
// }
pub static GLOBAL_PONG_PAYLOAD: Lazy<Mutex<Pong::Pong_Payload>> = Lazy::new(|| {
    Mutex::new(Pong::Pong_Payload {
        Port: String::new(),
        Ip: String::new(),
        Num_files: 0,
        Num_kb: 0,
    })
});

pub struct MessagePath {
    stream: Option<TcpStream>,
    id: String,
}

static Message_Paths: Lazy<Mutex<Vec<MessagePath>>> = Lazy::new(|| Mutex::new(Vec::new()));

impl MessagePath {
    pub fn add_ping_path(stream: Option<TcpStream>, id: String) {
        let mut paths = Message_Paths.lock().unwrap(); // Lock the mutex
        paths.push(MessagePath { stream, id });
    }
    pub fn get_stream_by_id(id: &String) -> Option<TcpStream> {
        let paths = Message_Paths.lock().unwrap(); // Lock the mutex
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
    let mut streams_dup: Vec<Option<TcpStream>> = streams
        .iter_mut()
        .map(|stream| stream.as_mut().and_then(|s| s.try_clone().ok()))
        .collect();
    for stream_option in streams.iter_mut() {
        if let Some(stream) = stream_option {
            // Set non-blocking mode
            stream.set_nonblocking(true)?;

            let mut buff = [0; 2048];
            match stream.read(&mut buff) {
                Ok(bytes_read) if bytes_read > 0 => {
                    if let Ok(stream_clone) = stream.try_clone() {
                        // Set blocking mode back for handling
                        stream_clone.set_nonblocking(false)?;
                        if let Err(e) = HandleServent::handle_connection(
                            &mut streams_dup,
                            stream_clone,
                            bytes_read,
                            &buff,
                        ) {
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
    if args.len() < 4 {
        eprintln!("Usage: {} <ip:port> <connect_address> <file_paths_list>", args[0]);
        std::process::exit(1);
    }

    let (num_files, total_kb) = match HandleFiles::PathValidator::validate_and_store_file_paths(&args[3]) {
        Ok((num_files, total_kb)) => {
            println!("Validated {} file paths successfully, total size: {} KB", num_files, total_kb);
            (num_files, total_kb)
        },
        Err(e) => {
            eprintln!("Error validating file paths: {}", e);
            return Err(e);
        }
    };
    
    let listener = TcpListener::bind(args[1].clone())?;
    println!("Server listening on {}", args[1]);
    println!("servent id: {:?}", *SERVENT_ID);

    if let Ok(mut payload) = GLOBAL_PONG_PAYLOAD.lock() {
        // split first arg into IP and port
        let parts: Vec<&str> = args[1].split(':').collect();
        payload.Ip = parts[0].to_string();
        payload.Port = parts[1].to_string();
        
        payload.Num_files = num_files as u32;
        payload.Num_kb = total_kb as u32;
    }

    let listener = Arc::new(listener);
    let handle_listener = Arc::clone(&listener);

    let streams: Arc<Mutex<Vec<Option<TcpStream>>>> = Arc::new(Mutex::new(Vec::new()));
    let streams_clone1 = Arc::clone(&streams);
    let streams_clone2 = Arc::clone(&streams);

    // spawn stream checker thread
    let checker_handle = {
        let streams = Arc::clone(&streams);
        thread::spawn(move || loop {
            thread::sleep(time::Duration::from_secs(1));
            if let Ok(mut streams) = streams.lock() {
                if let Err(e) = check_streams(&mut streams) {
                    eprintln!("Error checking streams: {}", e);
                }
            }
        })
    };

    let handle = thread::spawn(move || {
        if args[2] != "-1" {
            match TcpStream::connect(args[2].clone()) {
                Ok(mut stream) => {
                    // let stream_dup=stream.try_clone();
                    let stream_temp=stream.try_clone();
                    if let Ok(mut streams) = streams_clone2.lock() {
                        streams.push(Some(stream_temp.expect("REASON")));
                    }
                    

                    HandleClient::handle_requests(Some(&mut stream),streams_clone2);
                    
                }
                Err(e) => {
                    eprintln!("Failed to connect to {}: {}", args[2], e);
                }
            }
        }
        else{
            HandleClient::handle_requests(None,streams_clone2);


        }
        
    });
    
    
    // handle incoming connections
    for stream in handle_listener.incoming() {
        match stream {
            Ok(stream) => {
                // println!("New connection: {:?}", stream);
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
    handle.join().unwrap();
    println!("Exiting...");
    Ok(())
}
