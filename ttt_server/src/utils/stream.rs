use anyhow::Result;
use std::{io::Read, net::TcpStream};

fn filter_zeroes(bytes: Vec<u8>) -> Vec<u8> {
    return bytes.iter().filter(|x| **x != 0).cloned().collect();
}

pub fn read_from_stream(stream: &mut TcpStream) -> Result<String> {
    let mut buffer = [0; 64];
    let mut vec_buffer = vec![];

    while stream.read(&mut buffer)? >= buffer.len() {
        for x in filter_zeroes(buffer.to_vec()) {
            vec_buffer.push(x);
        }

        buffer = [0; 64];
    }

    // last .read won't be fired in the while loop above
    for x in filter_zeroes(buffer.to_vec()) {
        vec_buffer.push(x);
    }

    return Ok(String::from_utf8(vec_buffer)?);
}
