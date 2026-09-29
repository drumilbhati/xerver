use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    println!("Accepted connection from: {}", stream.peer_addr()?);
    stream.set_nonblocking(true)?;

    let mut buffer = [0u8; 1024];
    loop {
        // Read incoming bytes into our mutable buffer
        match stream.read(&mut buffer) {
            Ok(0) => {
                // EOF, close the connection
                println!("Client disconnected cleanly.");
                break;
            }
            Ok(bytes_read) => {
                // We received actual data.
                stream.write(&buffer[..bytes_read])?;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // No data available right now.
                // The client is still connected, just idle!
                // We do NOT break or return an error.}
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    // Notice: We don't call close(stream).
    // When handle_client returns, 'stream' goes out of scope and its Drop
    // implementation automatically closes the Linux socket file descriptor!
    Ok(())
}

fn main() -> io::Result<()> {
    let port = 3030;
    let addr = format!("0.0.0.0:{}", port);

    // 1. Create socket, bind to address, and listen for connections
    let listener = TcpListener::bind(&addr)?;
    println!("Listening on {}", addr);

    // 2. Accept connections in a loop
    for stream_result in listener.incoming() {
        match stream_result {
            Ok(stream) => {
                // spawn a new thread and move the stream into it
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
    Ok(())
}
