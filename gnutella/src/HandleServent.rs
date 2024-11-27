use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;


use crate::HandleClient;
use crate::InitializeConn;
use crate::Messages;
use crate::PingPath;
use crate::Pong;

use crate::GLOBAL_PONG_PAYLOAD;
use crate::SERVENT_ID;

pub struct PongLogger;

impl PongLogger {
    /// Log a Pong message to pongs.txt
    pub fn log_pong(pong_payload: &Pong::Pong_Payload) -> Result<(), std::io::Error> {
        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Prepare log entry
        let log_entry = format!(
            "{}\tIP: {}\tPort: {}\tFiles: {}\tKB: {}\n",
            timestamp,
            pong_payload.Ip,
            pong_payload.Port,
            pong_payload.Num_files,
            pong_payload.Num_kb
        );
        let binding = [SERVENT_ID.to_string(),"_pongs.txt".to_string()].concat();

        // Open file in append mode, create if not exists
        let path = Path::new(&binding);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        // Write log entry
        file.write_all(log_entry.as_bytes())?;

        Ok(())
    }

    /// Thread-safe logging method using a global mutex
    pub fn thread_safe_log_pong(pong_payload: &Pong::Pong_Payload) {
        // Use a lazy_static mutex to ensure thread-safe file writing
        lazy_static! {
            static ref PONG_LOG_MUTEX: Mutex<()> = Mutex::new(());
        }

        // Acquire the mutex lock
        let _lock = PONG_LOG_MUTEX.lock().unwrap();

        // Log the pong
        if let Err(e) = Self::log_pong(pong_payload) {
            eprintln!("Failed to log pong: {}", e);
        }
    }
}

pub fn handle_connection(
    streams: &mut Vec<Option<TcpStream>>,
    mut stream: TcpStream,
    bytes_read: usize,
    buff: &[u8],
) -> Result<(), std::io::Error> {
    let mut current_buffer = &buff[..bytes_read];

    while !current_buffer.is_empty() {
        // Check if there's enough data to read the header
        // if current_buffer.len() < 23 {
        //     break;
        // }

        // Parse the header
        let response1 = match Messages::from_bytes(current_buffer) {
            Some(header) => header,
            None => break, // Invalid header, stop processing
        };

        if response1.get_payload_descriptor() == &Messages::Payload_type::Connect {
            InitializeConn::accept_conn(stream.try_clone()?);
            return Ok(());
        }

        // Calculate total message length
        let payload_length = response1.get_payload_length() as usize;
        let total_message_length = 23 + payload_length;

        // Check if we have the full message in the buffer
        // if current_buffer.len() < total_message_length {
        //     break; // Incomplete message, wait for more data
        // }

        // Extract payload

        let payload_buff = &current_buffer[23..total_message_length];

        // Process message based on type
        // println!("{:?}",response1);
        match response1.get_payload_descriptor() {
            Messages::Payload_type::Ping => {
                handle_ping_message(streams, &mut stream, &response1, payload_buff)?;
            }
            Messages::Payload_type::Pong => {
                handle_pong_message(&mut stream, &response1, payload_buff)?;
            }
            Messages::Payload_type::Connect
            | Messages::Payload_type::Push
            | Messages::Payload_type::Query
            | Messages::Payload_type::Query_Hit => todo!(),
        }

        // Move to the next message in the buffer
        current_buffer = &current_buffer[total_message_length..];
    }

    Ok(())
}

fn handle_ping_message(
    streams: &mut Vec<Option<TcpStream>>,
    current_stream: &mut TcpStream,
    header: &Messages::Header,
    payload_buff: &[u8],
) -> Result<(), std::io::Error> {
    let id = header.get_descriptor_id();
    let ttl = header.get_hops() + 2;

    // Send Pong response
    if let Ok(payload) = GLOBAL_PONG_PAYLOAD.lock() {
        Pong::send_pong(current_stream, payload.to_bytes(), &id, &ttl, 0);
    }

    // Forward ping if TTL allows
    if header.get_ttl() != 0 {
        PingPath::add_ping_path(Some(current_stream.try_clone()?), id.clone());

        for stream_option in streams.iter_mut() {
            if let Some(stream1) = stream_option {
                if let (Ok(addr1), Ok(addr2)) = (stream1.peer_addr(), current_stream.peer_addr()) {
                    if addr1 != addr2 {
                        HandleClient::send_ping(
                            stream1,
                            id.clone(),
                            header.get_ttl() - 1,
                            header.get_hops() + 1,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn handle_pong_message(
    current_stream: &mut TcpStream,
    header: &Messages::Header,
    payload_buff: &[u8],
) -> Result<(), std::io::Error> {
    println!(
        "Pong message: {:?}",
        Pong::Pong_Payload::from_bytes(payload_buff)
    );
    // println!("Stream: {:?}", current_stream);

    let reverse_stream = PingPath::get_stream_by_id(header.get_descriptor_id());
    if header.get_ttl() !=0{
    if let Some(mut reverse_stream) = reverse_stream {
        // println!("Pong reverse stream: {:?}", reverse_stream);
        Pong::send_pong(
            &mut reverse_stream,
            payload_buff.to_vec(),
            &header.get_descriptor_id(),
            &(header.get_ttl() - 1),
            header.get_hops() + 1
        );
    }
    else{
        // println!("writing");
        PongLogger::thread_safe_log_pong(&Pong::Pong_Payload::from_bytes(payload_buff));
    }
    
    }
    Ok(())
}
