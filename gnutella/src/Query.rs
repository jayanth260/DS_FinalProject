use std::io::prelude::*;
use std::net::TcpStream;
use crate::Messages;
use crate::HandleFiles;
// use crate::total_count;

#[derive(Debug)]
pub struct Query_Payload {
    pub Search_Criteria: String,
    pub Min_Speed: u16,
}

impl Query_Payload {
    pub fn new(search_criteria: String, min_speed: u16) -> Query_Payload {
        Query_Payload {
            Search_Criteria: search_criteria,
            Min_Speed: min_speed,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Add minimum speed as 2-byte integer (big-endian)
        bytes.extend_from_slice(&(self.Min_Speed as u16).to_be_bytes());
        
        // Add search criteria as bytes, terminated by a NUL
        bytes.extend_from_slice(self.Search_Criteria.as_bytes());
        bytes.push(0x00); // NUL terminator
        
        bytes
    }
    
    pub fn from_bytes(bytes: &[u8]) -> Self {
        // Extract minimum speed (first 2 bytes)
        let min_speed = u16::from_be_bytes([bytes[0], bytes[1]]);
        
        // Find the NUL terminator for search criteria
        let criteria_end = bytes[2..].iter()
            .position(|&x| x == 0x00)
            .map(|pos| pos + 2)
            .unwrap_or(bytes.len());
        
        // Extract search criteria
        let search_criteria = String::from_utf8_lossy(&bytes[2..criteria_end]).to_string();
        
        Self {
            Search_Criteria: search_criteria,
            Min_Speed: min_speed,
        }
    }
}

pub fn send_query(stream: &mut TcpStream, payload: &Query_Payload, id: &String, ttl: u8, hops: u8) {
    // println!("{:?},{:?}",stream,payload);
    
    // if let Ok(mut Count) = total_count.lock() {
    //     *Count+=1;
    // }
    let payload_bytes = payload.to_bytes();
    // println!("sending qeury");

    
    let header_bytes = Messages::Header::new(
        id.clone(),
        Messages::Payload_type::Query,
        ttl,
        hops,
        payload_bytes.len() as u32
    ).to_bytes();

    let combined_bytes = [header_bytes, payload_bytes].concat();
    
    stream.write_all(&combined_bytes).expect("Failed to send query");
}

pub fn search(query: Query_Payload) -> Option<Vec<(u32, u32, String)>> {
    let filename_prefix = "filename";
    if let Some(filename_match) = query.Search_Criteria.strip_prefix(filename_prefix) {
        let filename = filename_match.trim();
        
        let search_results = HandleFiles::PathValidator::is_file_shared(filename);
        
        if !search_results.is_empty() {
            let formatted_results: Vec<(u32, u32, String)> = search_results
                .into_iter()
                .map(|(size, index)| (
                    index,
                    size as u32,
                    filename.to_string()
                ))
                .collect();
            
            println!("Search results: {:?}", formatted_results);
            return Some(formatted_results);
        }
    }
    None
}