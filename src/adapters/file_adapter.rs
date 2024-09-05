use crate::{
    constants::BUF_SIZE,
    recorder::{Block, Input, Output},
    utils::{bytes_to_u32, u32_to_bytes},
};
use bus::{Bus, BusReader};
use chrono::Local;
use std::{
    fs::OpenOptions,
    io::{Error, Read, Write},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct FileAdapter {}

impl Output for FileAdapter {
    fn write(
        &self,
        block: Block,
        channel: &mut BusReader<([u8; BUF_SIZE], u32)>,
    ) -> Result<(), Error> {
        let date_string = format!("{:?}", Local::now().date_naive());
        let file_path = &block.file_path.replace("$date", date_string.as_str());

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file_path)
            .unwrap();

        let mut prev_time = Instant::now();

        loop {
            if let Ok((data, size)) = channel.recv() {
                // Add 4 time diff bytes
                let diff = u32_to_bytes(prev_time.elapsed().as_millis() as u32);
                prev_time = Instant::now();
                file.write_all(&diff).unwrap();

                // Write size header
                file.write_all(&u32_to_bytes(size)).unwrap();

                println!("Writing {:?} bytes to File", data.len());

                file.write_all(&data[0..size as usize]).unwrap();
            }
        }
    }
}

impl Input for FileAdapter {
    fn read(&self, block: Block, channel: &mut Bus<([u8; BUF_SIZE], u32)>) -> Result<(), Error> {
        let date_string = format!("{:?}", Local::now().date_naive());
        let file_path = &block.file_path.replace("$date", date_string.as_str());

        let mut file = OpenOptions::new()
            .read(true)
            .open(file_path)
            .expect("cannot open file");

        let mut pos: usize = 0;

        loop {
            if pos == file.metadata().unwrap().len() as usize {
                thread::sleep(Duration::from_millis(500));
                continue;
            }

            // If file is in timed format than remove first 4 bytes
            let mut diff_buf = [0; 4];
            pos += file.read(&mut diff_buf).unwrap();

            // Size header
            let mut size_buff = [0; 4];
            pos += file.read(&mut size_buff).unwrap();
            let size = bytes_to_u32(size_buff);

            if block.play_timed {
                let diff = bytes_to_u32(diff_buf);
                println!("Sleeping for {} ms", diff);
                thread::sleep(Duration::from_millis((diff as f64 * block.speed_multiplier) as u64));
            }

            let mut buf = [0; BUF_SIZE];
            file.read_exact(&mut buf[..size as usize]).unwrap();
            pos += size as usize;

            println!("Reading {} bytes from File", size);
            channel.broadcast((buf, size));
        }
    }
}
