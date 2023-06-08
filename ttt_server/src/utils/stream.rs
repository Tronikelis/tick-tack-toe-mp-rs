use anyhow::Result;
use std::{io::Read, net::TcpStream};

pub fn read_from_stream(stream: &mut TcpStream) -> Result<String> {
    let mut buffer = [0; 64];

    if stream.read(&mut buffer)? < buffer.len() {
        return Ok(String::from_utf8(buffer.to_vec())?);
    }

    let mut vec_buffer = vec![];
    while stream.read(&mut buffer)? > 0 {
        for x in buffer.iter().filter(|x| **x != 0) {
            vec_buffer.push(*x);
        }
    }

    return Ok(String::from_utf8(vec_buffer)?);
}
