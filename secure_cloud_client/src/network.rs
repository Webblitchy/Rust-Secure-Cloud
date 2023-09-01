use std::io::{Read, Write};
use std::net::TcpStream;
use crate::structs::RequestType;

const SERVER_ADDR: &str = "127.0.0.1:1234";

pub fn read_stream(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    const BLOC_SIZE: usize = 64; // TODO: check

    let mut received: Vec<u8> = vec![];

    let mut rx_bytes = [0u8; BLOC_SIZE];

    loop {
        let bytes_read = stream.read(&mut rx_bytes)?;
        received.extend_from_slice(&rx_bytes[..bytes_read]);

        if bytes_read < BLOC_SIZE {
            break;
        }
    }

    Ok(received)
}

pub fn write_stream(stream: &mut TcpStream, data: Vec<u8>) -> usize {
    let written = stream.write(data.as_slice()).unwrap();
    stream.flush().unwrap();
    written
}


fn connect() -> Option<TcpStream> {
    match TcpStream::connect(SERVER_ADDR) {
        Ok(stream) => {
            Some(stream)
        },
        Err(e) => {
            eprintln!("=> Failed to connect to server: {}", e);
            None
        }
    }
}


pub fn send_to_server(data: &mut Vec<u8>, request_type: RequestType, stream: Option<TcpStream>) -> Option<TcpStream> {
    data.push(request_type.clone() as u8); // dernier byte indique le type de requete

    let mut stream = match stream {
        None => connect()?,
        Some(stream) => stream.try_clone().unwrap()
    };

    stream.write(data.as_slice()).unwrap(); // envoi des données
    Some(stream)
}