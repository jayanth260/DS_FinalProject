use std::io::prelude::*;
use std::net::Ipv4Addr;
use std::net::TcpStream;
use crate::Messages;

use crate::SERVENT_ID;

#[derive(Debug, Clone)]
pub struct FileResult {
    pub file_index: u32,
    pub file_size: u32,
    pub file_name: String,
}

impl FileResult{
    pub fn new(index: u32, size: u32, name: String)-> FileResult{
        FileResult{
            file_index: index,
            file_size:size,
            file_name: name,
        }

    }
}

#[derive(Debug, Clone)]
pub struct QueryHit_Payload {
    pub Num_hits: u8,
    pub Port: String,
    pub Ip_address: String,
    pub Speed: u32,
    pub Results: Vec<FileResult>,
    pub Servent_id : String,
}

impl QueryHit_Payload {
    pub fn new(num_hits:u8, port: String, ip: String,speed:u32, results: Vec<FileResult>) -> QueryHit_Payload {
        QueryHit_Payload {
            Num_hits:num_hits,
            Port: port,
            Ip_address: ip,
            Speed: speed,
            Results: results,
            Servent_id : SERVENT_ID.as_bytes().iter().map(|byte| format!("{:08b}", byte)).collect(),
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        // Ensure we have at least 11 bytes for the initial header
        // if data.len() < 11 {
        //     return Err("Insufficient data for parsing network response");
        // }

        // Extract basic response details
        let servent_id_start = data.len() - 16*8;
    let servent_id_bytes = &data[servent_id_start..];
    let servent_id = String::from_utf8_lossy(servent_id_bytes).to_string();
        let num_hits = data[0];
        
        // Convert port to string
        let port = format!("{}", u16::from_be_bytes([data[1], data[2]]));
        
        // Convert IP address to string
        let ip_address = format!("{}.{}.{}.{}", 
            data[3], data[4], data[5], data[6]
        );
        
        // Parse speed
        let speed = u32::from_be_bytes([data[7], data[8], data[9], data[10]]);

        // Parse result set
        let mut results = Vec::with_capacity(num_hits as usize);
        let mut offset = 11;

        for _ in 0..num_hits {
            // Ensure we have enough bytes for a file result
            // if offset + 8 > data.len() {
            //     return Err("Insufficient data for file result");
            // }

            // Parse file index
            let file_index = u32::from_be_bytes([
                data[offset], 
                data[offset + 1], 
                data[offset + 2], 
                data[offset + 3]
            ]);
            offset += 4;

            // Parse file size
            let file_size = u32::from_be_bytes([
                data[offset], 
                data[offset + 1], 
                data[offset + 2], 
                data[offset + 3]
            ]);
            offset += 4;

            // Parse file name (null-terminated)
            let name_start = offset;
            while offset < data.len() && data[offset] != 0 {
                offset += 1;
            }

            // Convert filename to a string
            let file_name = String::from_utf8_lossy(&data[name_start..offset])
                .into_owned();

            // Move past the null terminator
            offset += 1;

            results.push(FileResult {
                file_index,
                file_size,
                file_name,
            });

            // Break if we've parsed all expected results or reached end of data
            if results.len() == num_hits as usize || offset >= data.len() {
                break;
            }
        }

        

        return QueryHit_Payload {
            Num_hits:num_hits,
            Port:port,
            Ip_address:ip_address,
            Speed:speed,
            Results:results,
            Servent_id: servent_id,

        };
    }

    // Optional: Method to serialize the response back to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
    
        // Add number of hits
        bytes.push(self.Num_hits);
    
        // Convert port to bytes
        let port_num: u16 = self.Port.parse().unwrap_or(0);
        bytes.extend_from_slice(&port_num.to_be_bytes());
    
        // Convert IP address to bytes
        let ip_parts: Vec<u8> = self.Ip_address
            .split('.')
            .map(|part| part.parse().unwrap_or(0))
            .collect();
        
        bytes.extend_from_slice(&ip_parts);
    
        // Add speed (big-endian)
        bytes.extend_from_slice(&self.Speed.to_be_bytes());
    
        // Add file results
        for result in &self.Results {
            // File index
            bytes.extend_from_slice(&result.file_index.to_be_bytes());
    
            // File size
            bytes.extend_from_slice(&result.file_size.to_be_bytes());
    
            // File name (null-terminated)
            bytes.extend_from_slice(result.file_name.as_bytes());
            bytes.push(0); // Null terminator
        }
        let servent_id_bytes = self.Servent_id.as_bytes();
        bytes.extend_from_slice(servent_id_bytes);
    
        bytes
    }
}

pub fn send_queryhit(stream: &mut TcpStream, payload: &QueryHit_Payload, id: &String, ttl: u8, hops: u8){
    let payload_bytes = payload.to_bytes();

    let header_bytes = Messages::Header::new(
        id.clone(),
        Messages::Payload_type::Query_Hit,
        ttl,
        hops,
        payload_bytes.len() as u32
    ).to_bytes();

    let combined_bytes = [header_bytes, payload_bytes].concat();
    
    stream.write_all(&combined_bytes).expect("Failed to send query hit");
}