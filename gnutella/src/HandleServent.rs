use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;


use crate::HandleFiles::SHARED_FILES;
use crate::HandleFiles::DOWNLOADED_FILES;
use crate::HandleClient;
use crate::InitializeConn;
use crate::Messages;
use crate::MessagePath;
use crate::Pong;
use crate::Query;
use crate::QueryHit;
use crate::Push;
use crate::HandleFiles;

use crate::GLOBAL_PONG_PAYLOAD;
use crate::SERVENT_ID;
use crate::GLOBAL_QUERYHIT_PAYLOADS;
use crate::TcpListener;



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
            Messages::Payload_type::Query => {
                handle_query_message(streams, &mut stream, &response1, payload_buff)?;
            }
            Messages::Payload_type::Query_Hit => {
                handle_queryhit_message(&mut stream, &response1, payload_buff)?;
            }
            Messages::Payload_type::Push => {
                handle_push_message(&mut stream, &response1, payload_buff)?;
            }
            Messages::Payload_type::Connect => todo!(),
        }

        // Move to the next message in the buffer
        current_buffer = &current_buffer[total_message_length..];
    }

    Ok(())
}

fn handle_query_message(
    streams: &mut Vec<Option<TcpStream>>,
    current_stream: &mut TcpStream,
    header: &Messages::Header,
    payload_buff: &[u8],
) -> Result<(), std::io::Error> {
    let id = header.get_descriptor_id();
    let ttl = header.get_hops() + 2;

    
    let search_result=Query::search(Query::Query_Payload::from_bytes(payload_buff));

    // println!("{:?}", search_result);
    if let Some(search_results) = search_result {
        // Process the results into QueryHit::FileResult
        let results: Vec<QueryHit::FileResult> = search_results.into_iter().map(|(file_index, file_size, file_name)| {
            QueryHit::FileResult {
                file_index,
                file_size,
                file_name,
            }
        }).collect();

        if !results.is_empty() {
            if let Ok(pong_payload) = GLOBAL_PONG_PAYLOAD.lock() {
                let payload = QueryHit::QueryHit_Payload::new(
                    results.len().try_into().unwrap(),
                    pong_payload.Port.clone(),
                    pong_payload.Ip.clone(),
                    300,
                    results,
                );

                QueryHit::send_queryhit(current_stream, &payload, id, ttl, 0);
            }
        }
    }
    

   

    // Forward ping if TTL allows
    if header.get_ttl() != 0 {
        MessagePath::add_ping_path(Some(current_stream.try_clone()?), id.clone());

        for stream_option in streams.iter_mut() {
            if let Some(stream1) = stream_option {
                if let (Ok(addr1), Ok(addr2)) = (stream1.peer_addr(), current_stream.peer_addr()) {
                    if addr1 != addr2 {
                        Query::send_query(
                            stream1,
                            &Query::Query_Payload::from_bytes(payload_buff),
                            &id.clone(),
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
        MessagePath::add_ping_path(Some(current_stream.try_clone()?), id.clone());

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
    // println!(
    //     "Pong message: {:?}",
    //     Pong::Pong_Payload::from_bytes(payload_buff)
    // );
    // println!("Stream: {:?}", current_stream);

    let reverse_stream = MessagePath::get_stream_by_id(header.get_descriptor_id());
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


fn handle_queryhit_message(
    current_stream: &mut TcpStream,
    header: &Messages::Header,
    payload_buff: &[u8],
) -> Result<(), std::io::Error> {
    
    // println!("Stream: {:?}", current_stream);

    let reverse_stream = MessagePath::get_stream_by_id(header.get_descriptor_id());
    if header.get_ttl() !=0{
    if let Some(mut reverse_stream) = reverse_stream {
        // println!("Pong reverse stream: {:?}", reverse_stream);
        QueryHit::send_queryhit(
            &mut reverse_stream,
            &QueryHit::QueryHit_Payload::from_bytes(payload_buff),
            &header.get_descriptor_id(),
            header.get_ttl() - 1,
            header.get_hops() + 1
        );
    }
    else{
        // handle downloads
        // PongLogger::thread_safe_log_pong(&Pong::Pong_Payload::from_bytes(payload_buff));
        // println!(
        //     "Queryhit : {:?}",
        //     QueryHit::QueryHit_Payload::from_bytes(payload_buff)
        // );
        let header_id = header.get_descriptor_id();
    
        // Update the global HashMap with the new payload
        if let Ok(mut global_queryhit_map) = GLOBAL_QUERYHIT_PAYLOADS.lock() {
            global_queryhit_map
                .entry(header_id.clone())
                .or_insert_with(Vec::new)
                .push(QueryHit::QueryHit_Payload::from_bytes(payload_buff).clone());
        }
        
    }
    
    }
    Ok(())
}

fn handle_push_message(
    current_stream: &mut TcpStream,
    header: &Messages::Header,
    payload_buff: &[u8],
) -> Result<(), std::io::Error> {
    let push_payload = Push::Push_Payload::from_bytes(payload_buff);
    
    let our_id_binary = SERVENT_ID.as_bytes()
        .iter()
        .map(|&byte| format!("{:08b}", byte))
        .collect::<String>();
    
    if push_payload.Servent_id == our_id_binary {
        println!("✅ Push request is for us!");
        
        // First check if this is a cached file
        let (file_path, original_metadata) = if push_payload.file_index >= 1000 {
            // Check downloaded/cached files
            if let Ok(downloaded_files) = DOWNLOADED_FILES.lock() {
                let cache_index = (push_payload.file_index - 1000) as usize;
                if let Some(metadata) = downloaded_files.get(cache_index) {
                    (Some(metadata.file_path.clone()), Some(metadata.clone()))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            // Check original shared files
            if let Ok(shared_files) = SHARED_FILES.lock() {
                (shared_files.get(push_payload.file_index as usize)
                    .map(|s| s.to_string()), None)
            } else {
                (None, None)
            }
        };

        if let Some(file_path) = file_path {
            println!("📂 Found requested file: {}", file_path);
            
            // When handling a cache validation request (Node C's response)
            if push_payload.is_cache_check {
                println!("📝 Processing cache validation request");
                if let Ok(metadata) = std::fs::metadata(&file_path) {
                    if let Ok(modified_time) = metadata.modified() {
                        let our_modified = modified_time
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        
                        // Connect back to Node B's validation port
                        if let Ok(mut transfer_stream) = TcpStream::connect(format!("{}:{}", 
                            push_payload.Ip_address, push_payload.Port)) {  // Use the validation port!
                            
                            println!("✅ Connected back to cache holder for validation response");
                            
                            if our_modified > push_payload.cache_modified_time {
                                println!("🔄 Cache is outdated, sending updated file");
                                // Send the updated file
                                if let Ok(content) = std::fs::read(&file_path) {
                                    let response = format!(
                                        "HTTP/1.0 200 OK\r\n\
                                        Content-Type: application/octet-stream\r\n\
                                        Content-Length: {}\r\n\
                                        Server: Gnutella\r\n\
                                        \r\n",
                                        content.len()
                                    );
                                    transfer_stream.write_all(response.as_bytes())?;
                                    transfer_stream.write_all(&content)?;
                                }
                            } else {
                                println!("✅ Cache is up to date");
                                let response = "HTTP/1.0 304 Not Modified\r\n\
                                    X-Cache-Status: current\r\n\
                                    Server: Gnutella\r\n\
                                    \r\n";
                                transfer_stream.write_all(response.as_bytes())?;
                            }
                        }
                    }
                }
                return Ok(());
            }
            
            // Node B's logic
            if let Some(metadata) = original_metadata {
                if !push_payload.is_cache_check {
                    println!("🔄 Validating cache with original source...");
                    
                    // Start listening for response from original source
                    let listener = TcpListener::bind("127.0.0.1:0")?;
                    let local_addr = listener.local_addr()?;
                    println!("🌐 Listening on port {} for cache validation", local_addr.port());
                    
                    // Send validation request to C
                    if let Ok(mut original_stream) = TcpStream::connect(
                        format!("{}:{}", metadata.original_host, metadata.original_port)
                    ) {
                        println!("✅ Connected to original source for validation");
                        
                        let validation_payload = Push::Push_Payload {
                            Servent_id: metadata.original_servent_id.clone(),
                            file_index: metadata.original_index,
                            Ip_address: "127.0.0.1".to_string(),
                            Port: local_addr.port().to_string(),
                            is_cache_check: true,
                            cache_modified_time: std::fs::metadata(&file_path)
                                .and_then(|m| m.modified())
                                .map(|t| t.duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs())
                                .unwrap_or(0),
                            requesting_ip: "127.0.0.1".to_string(),
                            requesting_port: local_addr.port().to_string(),
                        };
                        
                        Push::send_push(
                            &mut original_stream,
                            &validation_payload,
                            &Messages::generate_desid(),
                            7,
                            0
                        );

                        // Wait for validation response from C
                        if let Ok((mut transfer_stream, _)) = listener.accept() {
                            println!("📥 Received validation response from original source");
                            
                            let mut buffer = [0; 1024];
                            let mut response = Vec::new();
                            
                            loop {
                                match transfer_stream.read(&mut buffer) {
                                    Ok(0) => break,
                                    Ok(n) => response.extend_from_slice(&buffer[..n]),
                                    Err(e) => {
                                        println!("❌ Error reading validation response: {}", e);
                                        break;
                                    }
                                }
                            }
                            
                            let response_str = String::from_utf8_lossy(&response);
                            println!("🔍 Response from C: {}", response_str); // Log the response

                            if response_str.contains("200 OK") {
                                // Update cache with new content
                                if let Some(body_start) = response.windows(4)
                                    .position(|window| window == b"\r\n\r\n") {
                                    let content = &response[body_start + 4..];
                                    if !content.is_empty() {
                                        std::fs::write(&file_path, content)?;
                                        println!("✅ Cache updated with new content");
                                    }
                                }
                            } else {
                                println!("✅ Cache is already up to date");
                            }
                        } else {
                            println!("❌ Failed to receive response from original source");
                        }
                    } else {
                        println!("❌ Failed to connect to original source");
                    }

                    // After validation, send file to A in a separate connection
                    println!("📤 Sending file to original requester");
                    if let Ok(mut requester_stream) = TcpStream::connect(format!("{}:{}", 
                        push_payload.Ip_address, push_payload.Port)) {
                        
                        // Read the file content
                        if let Ok(content) = std::fs::read(&file_path) {
                            let mut buffer = [0; 1024];
                            requester_stream.read(&mut buffer)?;  // Read HTTP GET request
                            
                            // Send file with 200 OK
                            let file_response = format!(
                                "HTTP/1.0 200 OK\r\n\
                                Content-Type: application/octet-stream\r\n\
                                Content-Length: {}\r\n\
                                Server: Gnutella\r\n\
                                \r\n",
                                content.len()
                            );
                            
                            requester_stream.write_all(file_response.as_bytes())?;
                            requester_stream.write_all(&content)?;
                            println!("✅ File sent successfully to requester ({} bytes)", content.len());
                        } else {
                            println!("❌ Failed to read file content");
                        }
                    } else {
                        println!("❌ Failed to connect to requester");
                    }
                }
            }
            
            // Now send the file (either original or updated cache)
            if let Ok(mut transfer_stream) = TcpStream::connect(format!("{}:{}", 
                push_payload.Ip_address, push_payload.Port)) {
                println!("✅ Connected to requester at {}:{}", 
                    push_payload.Ip_address, push_payload.Port);
                
                // Read the latest version of the file
                if let Ok(content) = std::fs::read(&file_path) {
                    let mut buffer = [0; 1024];
                    transfer_stream.read(&mut buffer)?;
                    
                    let response = format!(
                        "HTTP/1.0 200 OK\r\n\
                        Content-Type: application/octet-stream\r\n\
                        Content-Length: {}\r\n\
                        Server: Gnutella\r\n\
                        \r\n",
                        content.len()
                    );
                    
                    transfer_stream.write_all(response.as_bytes())?;
                    transfer_stream.write_all(&content)?;
                    println!("✅ File sent successfully ({} bytes)", content.len());
                }
            } else {
                println!("❌ Failed to connect to requester");
            }
        }
        
        Ok(())
    } else {
        // Forward the Push message if TTL allows
        if header.get_ttl() != 0 {
            let reverse_stream = MessagePath::get_stream_by_id(header.get_descriptor_id());
            if let Some(mut reverse_stream) = reverse_stream {
                Push::send_push(
                    &mut reverse_stream,
                    &push_payload,
                    header.get_descriptor_id(),
                    header.get_ttl() - 1,
                    header.get_hops() + 1
                );
            }
        }
        Ok(())
    }
}

// Helper function to handle the actual file transfer
fn handle_file_transfer(file_path: &str, push_payload: &Push::Push_Payload) -> Result<(), std::io::Error> {
    println!("🔍 Found file: {} (index: {})", file_path, push_payload.file_index);
    println!("📤 Connecting to requester at {}:{}", push_payload.Ip_address, push_payload.Port);
    
    // Connect to requester and send file
    if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", 
        push_payload.Ip_address, push_payload.Port)) {
        println!("✅ Connected to requester, waiting for HTTP request");
        
        // Read HTTP request
        let mut request = Vec::new();
        let mut buffer = [0; 1024];
        
        match stream.read(&mut buffer) {
            Ok(n) => {
                request.extend_from_slice(&buffer[..n]);
                println!("📥 Received HTTP request:\n{}", String::from_utf8_lossy(&request));
                
                // Read the file
                match std::fs::read(file_path) {
                    Ok(file_content) => {
                        // Send HTTP response with file
                        let response = format!(
                            "HTTP/1.0 200 OK\r\n\
                            Content-Type: application/octet-stream\r\n\
                            Content-Length: {}\r\n\
                            Server: Gnutella\r\n\
                            \r\n",
                            file_content.len()
                        );
                        
                        println!("📤 Sending response header:\n{}", response);
                        stream.write_all(response.as_bytes())?;
                        stream.write_all(&file_content)?;
                        println!("✅ File sent successfully ({} bytes)", file_content.len());
                    },
                    Err(e) => {
                        println!("❌ Failed to read file: {}", e);
                        let error_response = "HTTP/1.0 404 Not Found\r\n\r\n";
                        stream.write_all(error_response.as_bytes())?;
                    }
                }
            },
            Err(e) => {
                println!("❌ Failed to read HTTP request: {}", e);
            }
        }
    } else {
        println!("❌ Failed to connect to requester");
    }
    
    Ok(())
}
