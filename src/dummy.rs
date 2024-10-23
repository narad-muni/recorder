use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut client_stream: TcpStream, target_addr: &str) -> std::io::Result<()> {
    // Connect to guthib (or the target server)
    let mut server_stream = TcpStream::connect(target_addr)?;

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

fn main() -> std::io::Result<()> {
    let listen_addr = "127.0.0.1:8181";
    let target_addr = "127.0.0.1:8000";  // Targeting HTTP for guthib

    let listener = TcpListener::bind(listen_addr)?;
    println!("Listening on {}", listen_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(client_stream) => {
                let target_addr = target_addr.to_string();
                thread::spawn(move || {
                    if let Err(e) = handle_client(client_stream, &target_addr) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}
