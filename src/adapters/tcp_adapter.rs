use std::{
    io::{BufReader, Read, Write},
    net::TcpListener,
};

use bus::{Bus, BusReader};

use crate::{
    constants::BUF_SIZE,
    recorder::{Block, Input, Output},
};

#[derive(Debug)]
pub struct TcpAdapter {}

impl Input for TcpAdapter {
    fn read(
        &self,
        block: Block,
        channel: &mut Bus<([u8; BUF_SIZE], u32)>,
    ) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind((block.bind_ip, block.bind_port)).unwrap();

        while let Ok((conn, _)) = listener.accept() {
            let mut reader = BufReader::new(conn);

            loop {
                let mut buf = [0; BUF_SIZE];
                let result = reader.read(&mut buf);
                let length = result.as_ref().unwrap();

                if result.is_err() || *length == 0 {
                    break;
                }

                println!("Reading {:?} bytes from Tcp", length);
                channel.broadcast((buf, *length as u32));
            }
        }

        println!("Error while connecting");

        Ok(())
    }
}

impl Output for TcpAdapter {
    fn write(
        &self,
        block: Block,
        channel: &mut BusReader<([u8; BUF_SIZE], u32)>,
    ) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind((block.bind_ip, block.bind_port)).unwrap();

        while let Ok((mut conn, _)) = listener.accept() {
            if let Ok((data, size)) = channel.recv() {
                println!("Writing {:?} bytes to tcp", size);
                conn.write(&data).unwrap();
            }
        }

        println!("Error while connecting");

        Ok(())
    }
}
