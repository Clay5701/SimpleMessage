use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

// Main Entry Point
fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?; // Listens for incomming connections
    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new())); // Shared vector of connected clients
    println!("Listening on 127.0.0.1:7878...");

    // Accept incoming connections and spawn a new thread for each
    for stream in listener.incoming() {
        let stream = stream?;
        let clients = Arc::clone(&clients); // Clone the Arc for the thread
        println!("New connection from: {}", stream.peer_addr()?);

        thread::spawn(move || {
            let stream_clone = stream.try_clone().unwrap(); // Clone the stream for clients vector
            clients.lock().unwrap().push(stream_clone);

            handle_client(stream, clients);
        });
    }

    Ok(())
}

// Handles an individual client connection
fn handle_client(stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) {
    let my_addr = stream.peer_addr().unwrap(); // Store the client's address for later use
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let mut client_guard = clients.lock().unwrap();
                let mut i = 0;

                // Broadcast the message to all connected clients, except the sender
                // Remove clients that fail to send the message
                while i < client_guard.len() {
                    let client = &mut client_guard[i];
                    if client.write_all(line.as_bytes()).is_err() {
                        client_guard.remove(i);
                        continue;
                    }
                    i += 1;
                }
            }
            Err(e) => {
                println!("Client {} disconnected: {}", my_addr, e);
                break;
            }
        }
    }
}
