use std::{
    io::{BufReader, Read},
    net::SocketAddrV4,
};

use bus::{Bus, BusReader};
use socket2::{Domain, Protocol, Socket, Type};

use crate::{
    constants::BUF_SIZE,
    recorder::{Block, Input, Output},
};

#[derive(Debug)]
pub struct TcpClientAdapter {}

impl Input for TcpClientAdapter {
    fn read(
        &self,
        block: Block,
        channel: &mut Bus<([u8; BUF_SIZE], u32)>,
    ) -> Result<(), std::io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
        // socket
        //     .bind(&SocketAddrV4::new(block.bind_ip.parse().unwrap(), block.bind_port).into())
        //     .unwrap();
        socket
            .connect(&SocketAddrV4::new(block.source_ip.parse().unwrap(), block.source_port).into())
            .unwrap();

        let mut reader = BufReader::new(socket);

        loop {
            let mut buf = [0; BUF_SIZE];
            let result = reader.read(&mut buf);
            let length = result.as_ref().unwrap();

            if result.is_err() || *length == 0 {
                break;
            }

            #[cfg(debug_assertions)]
            println!("Reading {:?} bytes from Tcp", length);
            channel.broadcast((buf, *length as u32));
        }

        println!("Error while connecting");

        Ok(())
    }
}

impl Output for TcpClientAdapter {
    fn write(
        &self,
        block: Block,
        channel: &mut BusReader<([u8; BUF_SIZE], u32)>,
    ) -> Result<(), std::io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
        // socket
        //     .bind(&SocketAddrV4::new(block.bind_ip.parse().unwrap(), block.bind_port).into())
        //     .unwrap();
        socket
            .connect(&SocketAddrV4::new(block.source_ip.parse().unwrap(), block.source_port).into())
            .unwrap();

        while let Ok((data, size)) = channel.recv() {
            #[cfg(debug_assertions)]
            println!("Writing {:?} bytes to tcp", size);
            socket.send(&data).unwrap();
        }

        println!("Error while connecting");

        Ok(())
    }
}
