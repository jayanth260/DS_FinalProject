use std::io::prelude::*;
use std::net::TcpStream;
use crate::Messages;
use crate::HandleFiles;
use std::net::TcpListener;
use crate::Push;

#[derive(Debug)]
pub struct Push_Payload {
    pub Servent_id: String,
    pub file_index: u32,
    pub Ip_address: String,
    pub Port: String,
}

impl Push_Payload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Convert UUID string to bytes using the same logic as QueryHit
        let bytes_vec: Vec<u8> = self.Servent_id
            .as_bytes()
            .chunks(8)
            .map(|chunk| {
                let chunk_str = std::str::from_utf8(chunk).unwrap();
                u8::from_str_radix(chunk_str, 2).unwrap()
            })
            .collect();
        bytes.extend_from_slice(&bytes_vec);
        
        // Add File Index (4 bytes)
        bytes.extend_from_slice(&self.file_index.to_be_bytes());
        
        // Add IP Address (4 bytes)
        for octet in self.Ip_address.split('.') {
            bytes.push(octet.parse::<u8>().unwrap_or(0));
        }
        
        // Add Port (2 bytes)
        let port_num: u16 = self.Port.parse().unwrap_or(0);
        bytes.extend_from_slice(&port_num.to_be_bytes());
        
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        // Use the same logic as QueryHit to convert bytes back to UUID string
        let servent_id = bytes[0..16]
            .iter()
            .map(|&byte| format!("{:08b}", byte))
            .collect::<String>();

        let file_index = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let ip_address = format!("{}.{}.{}.{}", bytes[20], bytes[21], bytes[22], bytes[23]);
        let port = format!("{}", u16::from_be_bytes([bytes[24], bytes[25]]));

        Push_Payload {
            Servent_id: servent_id,
            file_index,
            Ip_address: ip_address,
            Port: port,
        }
    }

    pub fn download_file(&self, filename: &str) -> std::io::Result<()> {
        println!("Starting download process...");
        std::fs::create_dir_all("downloads")?;
        
        // Start listening for the incoming file transfer
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let local_addr = listener.local_addr()?;
        println!("🌐 Listening on port {} for file transfer", local_addr.port());
        
        // Create Push request with our listening port
        let push_payload = Push_Payload {
            Servent_id: self.Servent_id.clone(),
            file_index: self.file_index,
            Ip_address: "127.0.0.1".to_string(),
            Port: local_addr.port().to_string(), // The port we're listening on
        };
        
        // Connect to Gnutella network and send Push request
        println!("Connecting to Gnutella network at {}:{}", self.Ip_address, self.Port);
        let mut gnutella_stream = TcpStream::connect(format!("{}:{}", self.Ip_address, self.Port))?;
        Push::send_push(&mut gnutella_stream, &push_payload, &Messages::generate_desid(), 7, 0);
        println!("Push request sent, waiting for file sender to connect...");
        
        // Wait for the file sender to connect
        match listener.accept() {
            Ok((mut transfer_stream, addr)) => {
                println!("✅ File sender connected from {}", addr);
                
                // Send HTTP GET request
                let request = format!(
                    "GET /get/{}/{} HTTP/1.0\r\n\
                    Host: {}:{}\r\n\
                    Connection: Close\r\n\
                    User-Agent: Gnutella\r\n\
                    \r\n",
                    self.file_index,
                    filename,
                    self.Ip_address,
                    self.Port
                );
                
                println!("📤 Sending HTTP request:\n{}", request);
                transfer_stream.write_all(request.as_bytes())?;
                
                // Read the response
                let mut response = Vec::new();
                let mut buffer = [0; 4096];
                
                loop {
                    match transfer_stream.read(&mut buffer) {
                        Ok(0) => break, // Connection closed
                        Ok(n) => {
                            println!("📥 Received {} bytes", n);
                            response.extend_from_slice(&buffer[..n]);
                        },
                        Err(e) => {
                            println!("❌ Error reading response: {}", e);
                            return Err(e);
                        }
                    }
                }
                
                if response.is_empty() {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, "Empty response"));
                }
                
                // Parse and save the file
                if let Some(body_start) = find_body_start(&response) {
                    let file_path = format!("downloads/{}", filename);
                    std::fs::write(&file_path, &response[body_start..])?;
                    println!("✅ File saved to: {}", std::fs::canonicalize(&file_path)?.display());
                    Ok(())
                } else {
                    println!("❌ Invalid HTTP response");
                    println!("Response received: {}", String::from_utf8_lossy(&response));
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid HTTP response"))
                }
            },
            Err(e) => {
                println!("❌ Failed to accept connection: {}", e);
                Err(e)
            }
        }
    }
}

// Helper function to find the start of HTTP body
fn find_body_start(response: &[u8]) -> Option<usize> {
    let response_str = String::from_utf8_lossy(response);
    println!("Looking for body in response:\n{}", response_str);
    
    // Find double CRLF that separates headers from body
    if let Some(idx) = response_str.find("\r\n\r\n") {
        Some(idx + 4)
    } else {
        None
    }
}

pub fn send_push(stream: &mut TcpStream, payload: &Push_Payload, id: &String, ttl: u8, hops: u8) {
    let payload_bytes = payload.to_bytes();
    
    let header_bytes = Messages::Header::new(
        id.clone(),
        Messages::Payload_type::Push,
        ttl,
        hops,
        payload_bytes.len() as u32
    ).to_bytes();

    let combined_bytes = [header_bytes, payload_bytes].concat();
    stream.write_all(&combined_bytes).expect("Failed to send push request");
}

pub fn handle_push_request(payload: Push_Payload, filename: &str) -> std::io::Result<()> {
    println!("📥 Received Push request for file: {}", filename);
    println!("   From IP: {}, Port: {}", payload.Ip_address, payload.Port);
    
    match payload.download_file(filename) {
        Ok(()) => {
            println!("✅ File download completed successfully");
            Ok(())
        },
        Err(e) => {
            eprintln!("❌ Error downloading file: {}", e);
            Err(e)
        }
    }
}