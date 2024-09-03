use std::fs::File;
use std::io::stdin;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    // Ask the user for the number of bytes to read
    println!("Enter the number of bytes to read:");

    // Get the user input
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read input");
    let n: usize = input.trim().parse().expect("Please enter a valid number");

    // Open the file
    let mut file = File::open("example.txt")?;

    // Create a buffer of 512 bytes
    let mut buffer = [0; 512];

    // Read n bytes into the buffer
    let bytes_read = file.read(&mut buffer[..n])?;

    // Output the read bytes (optional)
    println!("Read {} bytes: {:?}", bytes_read, &buffer[..bytes_read]);

    Ok(())
}
