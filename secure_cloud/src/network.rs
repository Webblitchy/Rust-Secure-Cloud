use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};

pub fn read_stream(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    const BLOC_SIZE: usize = 64;

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
    //stream.flush();
    written
}

pub fn shutdown_stream(stream: &mut TcpStream) {
    stream.shutdown(Shutdown::Both).unwrap();
}