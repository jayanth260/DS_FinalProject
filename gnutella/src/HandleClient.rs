use crate::InitializeConn;
use crate::Messages;
use std::io::{self, prelude::*};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use prettytable::{Table, Row, Cell};
use crate::Query;
use uuid::Uuid;

use crate::QueryHit;
// use crate::total_count;
// use crate::query_hit;

// const files: [&str; 15]=["file1.txt","file2.txt","file3.txt","file4.txt","file5.txt","file6.txt","file7.txt","file8.txt",
// "file9.txt",
// "file10.txt",
// "file11.txt",
// "file12.txt",
// "file13.txt",
// "file14.txt",
// "file15.txt"];


use crate::GLOBAL_QUERYHIT_PAYLOADS;
pub fn format_query_hits(payloads: Vec<QueryHit::QueryHit_Payload>) {
    // Create a new table
    let mut table = Table::new();

    // Add a header row
    table.add_row(Row::new(vec![
        Cell::new("Servent ID"),
        Cell::new("File Name"),
        Cell::new("File Size (B)"),
        Cell::new("File Index"),
        Cell::new("IP Address"),
        Cell::new("Port"),
        Cell::new("Speed (Kbps)"),
    ]));

    // Add rows for each payload and its results
    for payload in payloads {
        for result in &payload.Results {
            
            let  bytes: Vec<u8> = payload.Servent_id
            .as_bytes()
            .chunks(8)
            .map(|chunk| {
                let chunk_str = std::str::from_utf8(chunk).unwrap();
                u8::from_str_radix(chunk_str, 2).unwrap()
            })
            .collect();

            table.add_row(Row::new(vec![
                Cell::new(&Uuid::from_slice(&bytes).map_or_else(|_| "Invalid UUID".to_string(), |uuid| uuid.to_string())),
                Cell::new(&result.file_name),

                Cell::new(&(result.file_size).to_string()), // Convert file size to KB
                Cell::new(&(result.file_index).to_string()),
                Cell::new(&payload.Ip_address),
                Cell::new(&payload.Port),
                Cell::new(&payload.Speed.to_string()),
            ]));
        }
    }

    // Print the table
    table.printstd();
}

// Helper functions to interact with the global QueryHit payloads
pub fn get_queryhits_by_header_id(header_id: &str) -> Vec<QueryHit::QueryHit_Payload> {
    if let Ok(global_queryhit_map) = GLOBAL_QUERYHIT_PAYLOADS.lock() {
        global_queryhit_map
            .get(header_id)
            .cloned()
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn clear_queryhits_by_header_id(header_id: &str) {
    if let Ok(mut global_queryhit_map) = GLOBAL_QUERYHIT_PAYLOADS.lock() {
        global_queryhit_map.remove(header_id);
    }
}

pub fn clear_all_queryhits() {
    if let Ok(mut global_queryhit_map) = GLOBAL_QUERYHIT_PAYLOADS.lock() {
        global_queryhit_map.clear();
    }
}


pub fn handle_requests(
    stream:Option<&mut TcpStream>,
    streams: Arc<Mutex<Vec<Option<TcpStream>>>>
) -> io::Result<()> {
    // Send a connection request
    match stream{
        Some(stream)=>{
    let response = InitializeConn::request_conn(stream)?;
    
    // Check if server accepts connection
    if response.contains("200 OK") {
        println!("🟢 Connection successful!");
        send_ping(stream, Messages::generate_desid(), 2, 0)?;
        
    } else {
        println!("❌ Connection unsuccessful. Check server status.");
    }
        },
        None=>{
           
        }
}
loop {
    // Clear menu
    println!("\n===== Gnutella-like P2P File Search =====");
    println!("1. Search for a file");
    println!("2. Exit");
    // println!("3. all");
    
    
    print!("Choose an option (1-2): ");
    io::stdout().flush().unwrap();
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    
    match choice.trim() {
        "1" => {
            // File search logic
            print!("Enter filename to search (case-sensitive): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let search_criteria = input.trim();
            
            if search_criteria.is_empty() {
                println!("❌ Search term cannot be empty!");
                continue;
            }
            
            let full_search_criteria = ["filename ".to_string(), search_criteria.to_string()].concat();
            let query_payload = Query::Query_Payload::new(full_search_criteria, 250);
            
            println!("🔍 Searching for: {}", search_criteria);
            
            // Broadcast query to all streams
            match send_query_to_all_streams(&streams, &query_payload) {
                Ok(header_id) => {
                    println!("✅ Query sent to {} connected streams", 
                        count_active_streams(&streams));
                    
                     
                    
                        thread::sleep(Duration::from_millis(5000));
          
                    // println!("{:?}",get_queryhits_by_header_id(&header_id));
                    format_query_hits(get_queryhits_by_header_id(&header_id))
                    // display_query_hits(&input, &header_id);
                    
                }
                Err(e) => {
                    eprintln!("❌ Error broadcasting query: {}", e);
                }
            }

            
            // Optional: Wait for user to continue
            print!("\nPress Enter to continue...");
            io::stdout().flush().unwrap();
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).unwrap();
        }
        "2" => {
            println!("Exiting...");
            // if let Ok(mut Count) = total_count.lock() {
            //     println!("{:?}",*Count);
            // }
            
            // if let Ok(queryhit_map) = GLOBAL_QUERYHIT_PAYLOADS.lock() {
            //     // Count the number of unique header IDs
            //     println!("{:?}",queryhit_map.keys().count()); // This counts distinct header_ids
            // } else {
            //     println!("0"); // Return 0 if the Mutex couldn't be locked
            // }

            break;
        }"3"=>{
            for file in files{
                let full_search_criteria = ["filename ".to_string(), file.to_string()].concat();
            let query_payload = Query::Query_Payload::new(full_search_criteria, 250);
            send_query_to_all_streams(&streams, &query_payload);
            thread::sleep(Duration::from_millis(5000));   
            }
        }
        _ => {
            println!("❌ Invalid option. Please choose 1 or 2.");
        }
        
    
}
}
    Ok(())
}

pub fn send_ping(
    stream: &mut TcpStream,
    id: String,
    ttl: u8,
    hops: u8,
) -> Result<(), std::io::Error> {

    // if let Ok(mut Count) = total_count.lock() {
    //     *Count+=1;
    // }
    
    
    let ping_header = Messages::Header::new(id, Messages::Payload_type::Ping, ttl, hops, 0);
    let ping_header_bytes = ping_header.to_bytes();
    stream.write_all(&ping_header_bytes)?;
    stream.flush()?;
    Ok(())
}


pub fn send_query_to_all_streams(
    streams: &Arc<Mutex<Vec<Option<TcpStream>>>>,
    query_payload: &Query::Query_Payload
) -> io::Result<String> {
    let mut streams_guard = streams.lock().map_err(|_|
        io::Error::new(io::ErrorKind::Other, "Could not acquire streams lock")
    )?;
    
    let mut successful_broadcasts = 0;
    
    // Generate a single descriptor ID for this query
    let descriptor_id = Messages::generate_desid();
    
    for stream_option in streams_guard.iter_mut() {
        if let Some(stream) = stream_option {
            match stream.try_clone() {
                Ok(mut stream_clone) => {
                    // Send query with the same descriptor ID
                    Query::send_query(&mut stream_clone, query_payload, &descriptor_id, 2, 0);
                    
                    successful_broadcasts += 1;
                }
                Err(e) => {
                    eprintln!("Failed to clone stream: {}", e);
                    *stream_option = None;
                }
            }
        }
    }
    println!("{:?}",successful_broadcasts);
    
    // Remove any streams that became None during processing
    streams_guard.retain(|stream| stream.is_some());
    
    if successful_broadcasts == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other, 
            "No active streams available to broadcast query"
        ));
    }
    
    Ok(descriptor_id)
}

// Utility function to total_count active streams
fn count_active_streams(
    streams: &Arc<Mutex<Vec<Option<TcpStream>>>>
) -> usize {
    streams.lock()
        .map(|guard| guard.iter().filter(|s| s.is_some()).count())
        .unwrap_or(0)
}