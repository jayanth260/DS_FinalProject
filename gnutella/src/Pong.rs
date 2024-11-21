use std::io::prelude::*;
use std::net::Ipv4Addr;
use std::net::TcpStream;



use crate::Messages;

#[derive(Debug)]
pub struct Pong_Payload{
    pub Port: String,
    pub Ip: String,
    pub Num_files: u32,
    pub Num_kb: u32
}


impl Pong_Payload{
    pub fn new(port: String, ip: String, num_files: u32, num_kb: u32)->Pong_Payload{
        Pong_Payload{
            Port: port,
            Ip: ip,
            Num_files: num_files,
            Num_kb: num_kb, 
        }
    }

    pub fn to_bytes(&self)->Vec<u8>{
        let mut bytes = Vec::with_capacity(14); 
        for chunk in self.Port.as_bytes().chunks(8){
        if let Ok(byte)= self.Port.parse::<u16>(){
            // println!("{}",byte);
            bytes.extend_from_slice(&byte.to_be_bytes());
        }
    }
        let ipparts: Vec<&str> = self.Ip.split('.').collect();    
    
    for (i, part) in ipparts.iter().enumerate() {
        if let Ok(value)= part.parse::<u8>(){
            bytes.push(value);
        }
        }
    bytes.extend_from_slice(&self.Num_files.to_be_bytes());
    bytes.extend_from_slice(&self.Num_kb.to_be_bytes());

    bytes

            }
    


    pub fn from_bytes(bytes: &[u8]) -> Self {
            
        
                // Extract and parse fields from the byte array
            let port = u16::from_be_bytes([bytes[0], bytes[1]]);
            let ip = Ipv4Addr::new(bytes[2], bytes[3], bytes[4], bytes[5]);
            let num_files = u32::from_be_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
            let num_kb = u32::from_be_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
        
                Self {
                    Port:port.to_string(),
                    Ip:ip.to_string(),
                    Num_files: num_files,
                    Num_kb: num_kb,
                }
            }
}


pub fn send_pong(stream: &mut TcpStream,payload:Vec<u8>, id: &String, ttl: &u8, hops: u8){
    
        // println!("{:?}", payload.Port);
        // let payload_bytes= payload.to_bytes();
        let header_bytes= Messages::Header::new(
            id.clone(),
            Messages::Payload_type::Pong,
            ttl.clone(),
            hops,
            14
        ).to_bytes();
        let combined_bytes = [header_bytes, payload].concat();
        println!("{:?}",combined_bytes);

        stream.write_all(&combined_bytes);


    

} 


