use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, thread};
use bus::Bus;

use crate::{
    constants::BUF_SIZE,
    recorder::{Block, Input},
};

fn handle_conn(mut client_stream: TcpStream, source_ip: String, source_port: u16) -> std::io::Result<()> {
    // Connect to guthib (or the target server)
    let mut server_stream = TcpStream::connect((source_ip, source_port))?;

    // Spawn a thread to handle client to server forwarding
    let mut server_stream_clone = server_stream.try_clone()?;
    let mut client_stream_clone = client_stream.try_clone()?;
    let client_to_server = thread::spawn(move || {
        let mut buffer = [0; 4096];
        while let Ok(bytes_read) = client_stream_clone.read(&mut buffer) {
            if bytes_read == 0 { break; }
            if server_stream_clone.write_all(&buffer[..bytes_read]).is_err() {
                break;
            }
        }
    });

    // Handle server to client forwarding
    let mut buffer = [0; 4096];
    while let Ok(bytes_read) = server_stream.read(&mut buffer) {
        if bytes_read == 0 { break; }
        if client_stream.write_all(&buffer[..bytes_read]).is_err() {
            break;
        }
    }

    // Wait for the client-to-server thread to finish
    client_to_server.join().ok();

    Ok(())
}

#[derive(Debug)]
pub struct TcpProxyAdapter {}

impl Input for TcpProxyAdapter {
    fn read(
        &self,
        block: Block,
        _channel: &mut Bus<([u8; BUF_SIZE], u32)>,
    ) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind((block.clone().bind_ip, block.clone().bind_port))?;

        for client in listener.incoming() {
            
            let block = block.clone();
            thread::spawn(move || {
                handle_conn(client.unwrap(), block.source_ip, block.source_port).unwrap();
            });
        }

        Ok(())
    }
}