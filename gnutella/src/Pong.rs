use std::io::prelude::*;
use std::net::Ipv4Addr;


#[derive(Debug)]
pub struct Pong_Payload{
    Port: String,
    Ip: String,
    Num_files: u32,
    Num_kb: u32
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
            println!("{}",byte);
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
    


    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
            if bytes.len() != 14 {
                    return Err("Invalid byte array length".to_string());
                }
        
                // Extract and parse fields from the byte array
            let port = u16::from_be_bytes([bytes[0], bytes[1]]);
            let ip = Ipv4Addr::new(bytes[2], bytes[3], bytes[4], bytes[5]);
            let num_files = u32::from_be_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]);
            let num_kb = u32::from_be_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
        
                Ok(Self {
                    Port:port.to_string(),
                    Ip:ip,
                    Num_files: num_files,
                    Num_kb: num_kb,
                })
            }
}

// pub fn from_bytes(bytes: &[u8])-> Pong_Payload{



// }