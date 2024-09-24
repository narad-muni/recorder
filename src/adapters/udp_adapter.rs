use std::{
    io::{BufReader, Read},
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use bus::{Bus, BusReader};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use crate::{
    constants::BUF_SIZE,
    recorder::{Block, Input, Output},
};

#[derive(Debug)]
pub struct UdpAdapter {}

impl Output for UdpAdapter {
    fn write(
        &self,
        block: Block,
        channel: &mut BusReader<([u8; BUF_SIZE], u32)>,
    ) -> Result<(), std::io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        socket.set_reuse_address(true).unwrap();
        socket
            .join_multicast_v4(
                &Ipv4Addr::from_str(&block.source_ip).unwrap(),
                &Ipv4Addr::UNSPECIFIED,
            )
            .unwrap();
        socket
            .bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into())
            .unwrap();

        socket
            .connect(&SockAddr::from(SocketAddrV4::new(
                block.source_ip.parse().unwrap(),
                block.source_port,
            )))
            .unwrap();

        loop {
            if let Ok((data, size)) = channel.recv() {
                println!("Writing {:?} bytes to udp", size);
                socket.send(&data[0..size as usize]).unwrap();
            }
        }
    }
}

impl Input for UdpAdapter {
    fn read(
        &self,
        block: Block,
        channel: &mut Bus<([u8; BUF_SIZE], u32)>,
    ) -> Result<(), std::io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        socket.set_reuse_address(true).unwrap();
        socket
            .join_multicast_v4(&block.source_ip.parse().unwrap(), &Ipv4Addr::from_str(&block.interface_ip).unwrap())
            .unwrap();
        socket
            .bind(&SocketAddrV4::new(Ipv4Addr::from_str(&block.bind_ip).unwrap(), block.source_port).into())
            .unwrap();

        let mut reader = BufReader::new(socket);

        loop {
            let mut buf = [0; BUF_SIZE];

            let response = reader.read(&mut buf);
            let length = response.as_ref().unwrap();

            if response.is_err() || *length == 0 {
                break;
            }

            channel.broadcast((buf, *length as u32));

            println!("Reading {:?} bytes from udp", length);
        }

        Ok(())
    }
}
