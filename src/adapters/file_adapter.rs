use crate::{
    constants::BUF_SIZE,
    recorder::{Block, Input, Output},
    utils::{bytes_to_u32, u32_to_bytes},
};
use bus::{Bus, BusReader};
use chrono::Local;
use std::{
    fs::OpenOptions,
    io::{stdin, BufReader, Error, Read, Seek, Write},
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
                if block.no_headers {
                    // Add 4 time diff bytes
                    let diff = u32_to_bytes(prev_time.elapsed().as_millis() as u32);
                    prev_time = Instant::now();
                    file.write_all(&diff).unwrap();

                    // Write size header
                    file.write_all(&u32_to_bytes(size)).unwrap();
                }

                #[cfg(debug_assertions)]
                println!("Writing {:?} bytes to File", size);

                file.write_all(&data[0..size as usize]).unwrap();
            }
        }
    }
}

impl Input for FileAdapter {
    fn read(&self, block: Block, channel: &mut Bus<([u8; BUF_SIZE], u32)>) -> Result<(), Error> {
        let date_string = format!("{:?}", Local::now().date_naive());
        let file_path = &block.file_path.replace("$date", date_string.as_str());

        let file = OpenOptions::new()
            .read(true)
            .open(file_path)
            .expect("cannot open file");

        // 128kb buffer
        let mut buf_reader = BufReader::with_capacity(131072, file);

        let mut count: i32 = 0;

        loop {
            // enter / 0 = 1 packet
            // 2 = 2 packets
            // -1 = infinite
            if block.controlled_play {

                if count == 0 {
                    let mut count_str = String::new();
                    println!("Waiting for key press");

                    stdin().read_line(&mut count_str).unwrap();

                    count = count_str.trim().parse().unwrap_or(1) - 1;
                }else if count > 0 {
                    count -= 1;
                }
            }

            // If file is in timed format than remove first 4 bytes
            let mut diff_buf = [0; 4];

            if let Ok(sz) = buf_reader.read(&mut diff_buf) {
                sz
            } else {
                if block.play_loop {
                    buf_reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                    continue;
                } else {
                    println!("File ended, waiting for changes");
                    thread::sleep(Duration::from_secs(2));
                    continue;
                };
            };

            // Size header
            let mut size_buff = [0; 4];
            
            if let Ok(sz) = buf_reader.read(&mut size_buff) {
                sz
            } else {
                if block.play_loop {
                    buf_reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                    continue;
                } else {
                    println!("File ended, waiting for changes");
                    thread::sleep(Duration::from_secs(2));
                    continue;
                };
            };

            let size = bytes_to_u32(size_buff);

            if size == 0 {

                if block.play_loop {
                    buf_reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                    continue;
                } else {
                    println!("File ended, waiting for changes");
                    thread::sleep(Duration::from_secs(2));
                }
            }

            if block.play_timed {
                let mut diff = bytes_to_u32(diff_buf);

                // If multiplier is more than 1, then lower limit of time diff should be 1 atleast
                // because multiplying by 0 is useless for slowing speed
                if block.speed_multiplier > 1.0 {
                    diff = diff.max(1);
                }

                #[cfg(debug_assertions)]
                println!("Sleeping for {} ms", diff as f64 * block.speed_multiplier);
                thread::sleep(Duration::from_millis(
                    (diff as f64 * block.speed_multiplier) as u64,
                ));
            }

            let mut buf = [0; BUF_SIZE];
            if let Err(_) = buf_reader.read_exact(&mut buf[..size as usize]) {
                if block.play_loop {
                    buf_reader.seek(std::io::SeekFrom::Start(0)).unwrap();
                    continue;
                } else {
                    println!("File ended, waiting for changes");
                    thread::sleep(Duration::from_secs(2));
                }
            }

            #[cfg(debug_assertions)]
            println!("Reading {} bytes from File", size);
            channel.broadcast((buf, size));
        }
    }
}
