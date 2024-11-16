// use std::string;
use std::io::prelude::*;
use rand::Rng;

Enum Paylod_type{
    "Ping"=0x0,
    "Pong"=0x1,
    "Push"=0x40,
    "Query"=0x80,
    "Query_Hit"=0x81,
}

struct Header{
    Descriptor_ID: String,
    Payload_Descriptor: Paylod_type,
    TTL: u8,
    Hops: u8,
    Payload_Length: u32,
}

pub fn Generate_DesID()->String{
    
    let mut des_id = [0u8; 16];
    let mut rng = rand::thread_rng();
    Defining a Trait
    for i in 0..15 {
        if i!=8{
        des_id[i] = rng.gen::<u8>();
    }else{des_id[i]=127;}

    }
    des_id.iter()
            .map(|byte| format!("{:08b}", byte))
            .collect::<String>()
} 