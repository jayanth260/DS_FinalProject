// use std::string;
use std::io::prelude::*;
use rand::Rng;

#[derive(PartialEq,Debug)]
pub enum Payload_type{
    Ping=0,
    Pong=1,
    Push=64,
    Query=128,
    Query_Hit=129,
    Connect=200,
}

#[derive(Debug)]
pub struct Header{
    Descriptor_ID: String,
    Payload_Descriptor: Payload_type,
    TTL: u8,
    Hops: u8,
    Payload_Length: u32,
}



impl Header {

    pub fn new(descriptor_id: String, payload_descriptor: Payload_type, ttl: u8, hops: u8, payload_length: u32) -> Header {
        Header {
            Descriptor_ID: descriptor_id,
            Payload_Descriptor: payload_descriptor,
            TTL: ttl,
            Hops: hops,
            Payload_Length: payload_length,
        }
    }
    pub fn get_descriptor_id(&self) -> &String {
        &self.Descriptor_ID
    }

    pub fn get_payload_descriptor(&self) -> &Payload_type {
        &self.Payload_Descriptor
    }

    pub fn get_ttl(&self) -> u8 {
        self.TTL
    }

    pub fn get_hops(&self) -> u8 {
        self.Hops
    }

    pub fn get_payload_length(&self) -> u32 {
        self.Payload_Length
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(23); 

        for chunk in self.Descriptor_ID.as_bytes().chunks(8) {
            if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 2) {
                bytes.push(byte);
            }
        }

        bytes.push(match self.Payload_Descriptor {
            Payload_type::Ping => 0x0,
            Payload_type::Pong => 0x1,
            Payload_type::Push => 0x40,
            Payload_type::Query => 0x80,
            Payload_type::Query_Hit => 0x81,
            Payload_type::Connect => 0xC8,
        });

        bytes.push(self.TTL);
        bytes.push(self.Hops);

        bytes.extend_from_slice(&self.Payload_Length.to_be_bytes());
        bytes
    }

    

}

pub fn from_bytes(bytes: &[u8]) -> Option<Header> {
    
    let mut response = String::from_utf8_lossy(&bytes).to_string();
    
    if response.contains("CONNECT") {
        return Some(Header {
            Descriptor_ID: "0".to_string(),
            Payload_Descriptor: Payload_type::Connect,
            TTL: 0,
            Hops: 0,
            Payload_Length: 0,
        });
    }
    
    if bytes.len() < 23 {
        return None;
    }

    let descriptor_id = bytes[0..16]
        .iter()
        .map(|byte| format!("{:08b}", byte))
        .collect::<String>();

    let payload_type = match bytes[16] {
        0x0 => Payload_type::Ping,
        0x1 => Payload_type::Pong,
        0x40 => Payload_type::Push,
        0x80 => Payload_type::Query,
        0x81 => Payload_type::Query_Hit,
        _ => return None,
    };

    let payload_length = u32::from_be_bytes([
        bytes[19], bytes[20], bytes[21], bytes[22]
    ]);

    Some(Header {
        Descriptor_ID: descriptor_id,
        Payload_Descriptor: payload_type,
        TTL: bytes[17],
        Hops: bytes[18],
        Payload_Length: payload_length,
    })
}

pub fn print_header(header: Header){
    println!("des_id: {}", header.Descriptor_ID);
    println!("payload type: 0x{:X}",header.Payload_Descriptor as u32);
    println!("TTL: {}",header.TTL);
    println!("Hops: {}",header.Hops);
    println!("payload length: {}", header.Payload_Length );

}

pub fn generate_desid()->String{
    
    let mut des_id = [0u8; 16];
    let mut rng = rand::thread_rng();
    for i in 0..15 {
        if i!=8{
        des_id[i] = rng.gen::<u8>();
    }else{des_id[i]=127;}

    }
    des_id.iter()
            .map(|byte| format!("{:08b}", byte))
            .collect::<String>()
}



