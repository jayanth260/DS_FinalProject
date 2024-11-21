use std::io::prelude::*;
use std::net::TcpStream;

use crate::InitializeConn;
use crate::Messages;
use crate::Pong;
use crate::HandleClient;
use crate::PingPath;

use crate::GLOBAL_PONG_PAYLOAD;


pub fn handle_connection(streams: &mut Vec<Option<TcpStream>>,mut stream: TcpStream,bytes_read: usize, buff: &[u8] ) -> Result<(), std::io::Error> {
    
    let mut buffer = buff.clone();
    // let bytes_read = stream.read(&mut buff)?;
    let mut response = String::from_utf8_lossy(&buff[..bytes_read]).to_string();
    // println!("{}", response);
    
    if response.contains("CONNECT") {
        InitializeConn::accept_conn(stream); // if connection is acceptable
    }
    else{

        let response1 : Messages::Header = Messages::from_bytes(buffer).expect("REASON");
        let payload_buff = &buffer[23..((response1.get_payload_length() as usize) + 23)];
        // if response1.get_descriptor_id()=="Pong"{
            
            
        // }
        if response1.get_payload_descriptor() == &Messages::Payload_type::Ping{
            let id= response1.get_descriptor_id();
            let ttl = response1.get_hops()+2;

            if let Ok(payload) = GLOBAL_PONG_PAYLOAD.lock() {
            Pong::send_pong(&mut stream,payload.to_bytes(), &id, &ttl,0);
            }

            if response1.get_ttl()!=0{
                
            PingPath::add_ping_path(Some(stream.try_clone().unwrap()), id.clone());
            

            for stream_option in streams.iter_mut(){
                if let Some(stream1) = stream_option {
                    if let (Ok(addr1), Ok(addr2)) = (stream1.peer_addr(), stream.peer_addr()) {
                        if addr1 != addr2 {
                            
                                // Extract TcpStream from Option
                                if let Some(stream_to_forward) = stream_option {
                                    HandleClient::send_ping(stream_to_forward,id.clone(), response1.get_ttl()-1, response1.get_hops()+1);
                                }
                            }
                        }
                    }
                }
            }
        }
        else if response1.get_payload_descriptor() == &Messages::Payload_type::Pong{
            println!("pong message:{:?}", Pong::Pong_Payload::from_bytes(payload_buff));
            println!("{:?}",stream);

            let reverse_stream= PingPath::get_stream_by_id(response1.get_descriptor_id()) ;
            if let Some(mut reverse_stream) = reverse_stream {
                println!("pong reverse stream: {:?}", reverse_stream);
                Pong::send_pong(
                    &mut reverse_stream,
                    payload_buff.to_vec(),
                    &response1.get_descriptor_id(),
                    &(response1.get_ttl() - 1),
                    response1.get_hops() + 1
                );
            }



        }
        

        // println!("{}",response1.get_descriptor_id());
        // let ip: String="127.0.0.1".to_string();
        // let port: String="5063".to_string();
        // let pong_message= Pong::Pong_Payload::new(port,ip,5,300);
        // println!("{:?}",pong_message);
        // println!("{:?}",Pong::Pong_Payload::from_bytes(&pong_message.to_bytes()));
    
    }


    Ok(())
}
